# frozen_string_literal: true

require 'spec_helper'
require 'rfmt/configuration'
require 'tempfile'
require 'fileutils'

RSpec.describe Rfmt::Configuration do
  describe '.discover' do
    let(:temp_dir) { Dir.mktmpdir }
    let(:original_dir) { Dir.pwd }

    before do
      Dir.chdir(temp_dir)
    end

    after do
      Dir.chdir(original_dir)
      FileUtils.rm_rf(temp_dir)
    end

    it 'discovers .rfmt.yml' do
      File.write('.rfmt.yml', "version: '1.0'")
      config = described_class.discover
      expect(config).to be_a(described_class)
    end

    it 'discovers .rfmt.yaml' do
      File.write('.rfmt.yaml', "version: '1.0'")
      config = described_class.discover
      expect(config).to be_a(described_class)
    end

    it 'discovers rfmt.yml' do
      File.write('rfmt.yml', "version: '1.0'")
      config = described_class.discover
      expect(config).to be_a(described_class)
    end

    it 'discovers rfmt.yaml' do
      File.write('rfmt.yaml', "version: '1.0'")
      config = described_class.discover
      expect(config).to be_a(described_class)
    end

    it 'returns default config when no file found' do
      config = described_class.discover
      expect(config.config).to include('version' => '1.0')
    end

    it 'prioritizes .rfmt.yml over other config files' do
      File.write('.rfmt.yml', "version: '2.0'")
      File.write('rfmt.yml', "version: '1.0'")
      config = described_class.discover
      expect(config.config['version']).to eq('2.0')
    end
  end

  describe '#initialize' do
    it 'uses default config when no options provided' do
      config = described_class.new
      expect(config.config).to include('version' => '1.0')
      expect(config.config['formatting']['line_length']).to eq(100)
      expect(config.config['formatting']['indent_width']).to eq(2)
    end

    it 'loads from file when file option provided' do
      temp_file = Tempfile.new(['config', '.yml'])
      temp_file.write("version: '2.0'\nformatting:\n  line_length: 120")
      temp_file.close

      config = described_class.new(file: temp_file.path)
      expect(config.config['version']).to eq('2.0')
      expect(config.config['formatting']['line_length']).to eq(120)

      temp_file.unlink
    end

    it 'merges custom options with defaults' do
      config = described_class.new('formatting' => { 'line_length' => 80 })
      expect(config.config['formatting']['line_length']).to eq(80)
      expect(config.config['formatting']['indent_width']).to eq(2) # Still has default
    end

    it 'validates positive line_length' do
      expect do
        described_class.new('formatting' => { 'line_length' => -1 })
      end.to raise_error(Rfmt::Configuration::ConfigError, 'line_length must be positive')
    end

    it 'validates positive indent_width' do
      expect do
        described_class.new('formatting' => { 'indent_width' => 0 })
      end.to raise_error(Rfmt::Configuration::ConfigError, 'indent_width must be positive')
    end
  end

  describe '#files_to_format' do
    let(:temp_dir) { Dir.mktmpdir }
    let(:original_dir) { Dir.pwd }

    before do
      # Create test file structure
      FileUtils.mkdir_p(File.join(temp_dir, 'lib'))
      FileUtils.mkdir_p(File.join(temp_dir, 'vendor'))
      FileUtils.mkdir_p(File.join(temp_dir, 'tmp'))

      File.write(File.join(temp_dir, 'lib', 'test.rb'), 'class Test; end')
      File.write(File.join(temp_dir, 'vendor', 'gem.rb'), 'class Gem; end')
      File.write(File.join(temp_dir, 'tmp', 'cache.rb'), 'class Cache; end')
      File.write(File.join(temp_dir, 'README.md'), '# README')
    end

    after do
      # Return to original directory before cleanup
      Dir.chdir(original_dir) rescue nil
      FileUtils.rm_rf(temp_dir)
    end

    it 'includes files matching include patterns' do
      config = described_class.new
      files = config.files_to_format(base_path: temp_dir)

      expect(files).to include(File.join(temp_dir, 'lib', 'test.rb'))
    end

    it 'excludes files matching exclude patterns' do
      config = described_class.new
      files = config.files_to_format(base_path: temp_dir)

      expect(files).not_to include(File.join(temp_dir, 'vendor', 'gem.rb'))
      expect(files).not_to include(File.join(temp_dir, 'tmp', 'cache.rb'))
    end

    it 'respects custom include patterns' do
      config = described_class.new('include' => ['lib/**/*.rb'])
      files = config.files_to_format(base_path: temp_dir)

      expect(files).to include(File.join(temp_dir, 'lib', 'test.rb'))
      expect(files.size).to eq(1)
    end

    it 'respects custom exclude patterns' do
      config = described_class.new('exclude' => ['lib/**/*'])
      files = config.files_to_format(base_path: temp_dir)

      expect(files).not_to include(File.join(temp_dir, 'lib', 'test.rb'))
    end

    it 'only returns files, not directories' do
      config = described_class.new
      files = config.files_to_format(base_path: temp_dir)

      files.each do |file|
        expect(File.file?(file)).to be true
      end
    end
  end

  describe '#formatting_config' do
    it 'returns formatting configuration' do
      config = described_class.new
      formatting = config.formatting_config

      expect(formatting).to be_a(Hash)
      expect(formatting).to include('line_length', 'indent_width', 'indent_style')
    end

    it 'returns custom formatting configuration' do
      config = described_class.new('formatting' => { 'line_length' => 80 })
      formatting = config.formatting_config

      expect(formatting['line_length']).to eq(80)
    end
  end

  describe 'configuration merging' do
    it 'deeply merges nested hashes' do
      config = described_class.new('formatting' => { 'line_length' => 80 })

      expect(config.config['formatting']['line_length']).to eq(80)
      expect(config.config['formatting']['indent_width']).to eq(2)
      expect(config.config['formatting']['indent_style']).to eq('spaces')
    end

    it 'overrides arrays instead of merging' do
      config = described_class.new('include' => ['custom/**/*.rb'])

      expect(config.config['include']).to eq(['custom/**/*.rb'])
    end
  end

  describe 'default configuration' do
    it 'has correct default values' do
      expect(described_class::DEFAULT_CONFIG).to include(
        'version' => '1.0',
        'formatting' => {
          'line_length' => 100,
          'indent_width' => 2,
          'indent_style' => 'spaces'
        },
        'include' => ['**/*.rb'],
        'exclude' => ['vendor/**/*', 'tmp/**/*', 'node_modules/**/*']
      )
    end

    it 'does not allow modification of default config' do
      expect(described_class::DEFAULT_CONFIG).to be_frozen
    end
  end
end
