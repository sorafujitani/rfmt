# frozen_string_literal: true

require 'spec_helper'
require 'rfmt'
require 'tmpdir'
require 'fileutils'

RSpec.describe 'Config-based formatting' do
  let(:source_code) do
    <<~RUBY
      class User
      def initialize(name)
      @name=name
      end
      end
    RUBY
  end

  around do |example|
    Dir.mktmpdir do |dir|
      Dir.chdir(dir) do
        example.run
      end
    end
  end

  describe 'formatting with custom indent_width' do
    it 'uses indent_width from rfmt.yml' do
      config_content = <<~YAML
        version: "1.0"
        formatting:
          line_length: 100
          indent_width: 4
          indent_style: "spaces"
      YAML
      File.write('rfmt.yml', config_content)

      formatted = Rfmt.format(source_code)

      # 4 spaces indentation should be used
      expect(formatted).to include('    def initialize(name)')
      expect(formatted).to include('        @name')
    end

    it 'uses indent_width from .rfmt.yml (backward compatibility)' do
      config_content = <<~YAML
        version: "1.0"
        formatting:
          line_length: 100
          indent_width: 4
          indent_style: "spaces"
      YAML
      File.write('.rfmt.yml', config_content)

      formatted = Rfmt.format(source_code)

      # 4 spaces indentation should be used
      expect(formatted).to include('    def initialize(name)')
      expect(formatted).to include('        @name')
    end
  end

  describe 'formatting with default config' do
    it 'uses 2 spaces when no config file exists' do
      formatted = Rfmt.format(source_code)

      # 2 spaces indentation should be used (default)
      expect(formatted).to include('  def initialize(name)')
      expect(formatted).to include('    @name')
    end
  end

  describe 'config file priority' do
    it 'prefers rfmt.yml over .rfmt.yml' do
      # Create both files with different configs
      File.write('rfmt.yml', <<~YAML)
        version: "1.0"
        formatting:
          indent_width: 3
          indent_style: "spaces"
      YAML

      File.write('.rfmt.yml', <<~YAML)
        version: "1.0"
        formatting:
          indent_width: 4
          indent_style: "spaces"
      YAML

      formatted = Rfmt.format(source_code)

      # Should use rfmt.yml (3 spaces)
      expect(formatted).to include('   def initialize(name)')
      expect(formatted).to include('      @name')
    end
  end

  describe 'config discovery in parent directories' do
    it 'finds config in parent directory' do
      # Create config in current directory
      config_content = <<~YAML
        version: "1.0"
        formatting:
          indent_width: 4
          indent_style: "spaces"
      YAML
      File.write('rfmt.yml', config_content)

      # Create subdirectory and format from there
      Dir.mkdir('subdir')
      Dir.chdir('subdir') do
        formatted = Rfmt.format(source_code)

        # Should still use parent's config (4 spaces)
        expect(formatted).to include('    def initialize(name)')
        expect(formatted).to include('        @name')
      end
    end
  end

  describe 'config applied to conditionals' do
    let(:conditional_code) do
      <<~RUBY
        def check(x)
        if x > 0
        puts "positive"
        else
        puts "not positive"
        end
        end
      RUBY
    end

    it 'applies custom indent_width to if/else blocks' do
      config_content = <<~YAML
        version: "1.0"
        formatting:
          indent_width: 4
          indent_style: "spaces"
      YAML
      File.write('rfmt.yml', config_content)

      formatted = Rfmt.format(conditional_code)

      # 4 spaces indentation for if/else blocks
      expect(formatted).to include('    if x > 0')
      expect(formatted).to include('        puts "positive"')
      expect(formatted).to include('    else')
      expect(formatted).to include('        puts "not positive"')
      expect(formatted).to include('    end')
    end

    it 'applies indent_style tabs to if/else blocks' do
      config_content = <<~YAML
        version: "1.0"
        formatting:
          indent_width: 1
          indent_style: "tabs"
      YAML
      File.write('rfmt.yml', config_content)

      formatted = Rfmt.format(conditional_code)

      # Tab indentation for if/else blocks
      expect(formatted).to include("\tif x > 0")
      expect(formatted).to include("\t\tputs \"positive\"")
      expect(formatted).to include("\telse")
      expect(formatted).to include("\t\tputs \"not positive\"")
    end
  end

  describe 'explicit config path' do
    it 'honors config_path over discovery' do
      File.write('rfmt.yml', <<~YAML)
        formatting:
          indent_width: 2
      YAML
      File.write('custom.yml', <<~YAML)
        formatting:
          indent_width: 4
      YAML

      formatted = Rfmt.format(source_code, config_path: 'custom.yml')

      expect(formatted.lines).to include("    def initialize(name)\n")
    end

    it 'raises loudly when the explicit path does not exist' do
      expect do
        Rfmt.format(source_code, config_path: 'missing.yml')
      end.to raise_error(Rfmt::Error, /Failed to read config file/)
    end

    it 'raises loudly when the explicit file is invalid' do
      File.write('broken.yml', <<~YAML)
        formatting:
          line_length: 20
      YAML

      # Discovery swallows broken files into defaults; an explicit path must not.
      expect do
        Rfmt.format(source_code, config_path: 'broken.yml')
      end.to raise_error(Rfmt::Error, /line_length/)
    end
  end

  describe 'discovery cache invalidation' do
    it 'picks up edits to the discovered config between in-process calls' do
      File.write('.rfmt.yml', <<~YAML)
        formatting:
          indent_width: 4
      YAML
      expect(Rfmt.format(source_code).lines).to include("    def initialize(name)\n")

      File.write('.rfmt.yml', <<~YAML)
        formatting:
          indent_width: 3
      YAML
      # Force a distinct mtime so the test does not depend on filesystem timestamp resolution
      File.utime(Time.now, Time.now + 2, '.rfmt.yml')

      expect(Rfmt.format(source_code).lines).to include("   def initialize(name)\n")
    end
  end
end
