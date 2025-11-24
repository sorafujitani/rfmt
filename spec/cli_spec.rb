# frozen_string_literal: true

require 'spec_helper'
require 'rfmt/cli'
require 'tempfile'
require 'fileutils'

RSpec.describe Rfmt::CLI do
  let(:cli) { described_class.new }
  let(:unformatted_code) do
    <<~RUBY
      class Foo
        def bar
          42
        end
      end
    RUBY
  end

  let(:formatted_code) do
    <<~RUBY.chomp
      class Foo
        def bar
          42
        end
      end
    RUBY
  end

  before do
    # Suppress CLI output during tests
    allow($stdout).to receive(:write)
    allow($stderr).to receive(:write)
  end

  describe '#version' do
    it 'displays version information' do
      expect { cli.version }.not_to raise_error
    end
  end

  describe '#format' do
    let(:temp_file) { Tempfile.new(['test', '.rb']) }

    before do
      temp_file.write(unformatted_code)
      temp_file.close
    end

    after do
      temp_file.unlink
    end

    context 'with write option' do
      it 'formats and writes to file' do
        cli.options = { write: true }
        cli.format(temp_file.path)

        formatted = File.read(temp_file.path)
        expect(formatted).to eq(formatted_code)
      end
    end

    context 'with no-write option' do
      it 'formats without writing to file' do
        cli.options = { write: false }
        original_content = File.read(temp_file.path)

        cli.format(temp_file.path)

        expect(File.read(temp_file.path)).to eq(original_content)
      end
    end

    context 'with check option' do
      it 'exits with code 1 when formatting is needed' do
        cli.options = { check: true, write: false }

        expect do
          cli.format(temp_file.path)
        end.to raise_error(SystemExit) do |error|
          expect(error.status).to eq(1)
        end
      end

      it 'exits with code 0 when file is already formatted' do
        # Write already formatted code
        File.write(temp_file.path, formatted_code)
        cli.options = { check: true, write: false }

        # Should not raise SystemExit since file is already formatted
        expect { cli.format(temp_file.path) }.not_to raise_error
      end
    end

    context 'with diff option' do
      it 'shows unified diff' do
        cli.options = { diff: true, write: false, diff_format: 'unified' }

        expect { cli.format(temp_file.path) }.not_to raise_error
      end

      it 'shows color diff' do
        cli.options = { diff: true, write: false, diff_format: 'color' }

        expect { cli.format(temp_file.path) }.not_to raise_error
      end

      it 'shows side_by_side diff' do
        cli.options = { diff: true, write: false, diff_format: 'side_by_side' }

        expect { cli.format(temp_file.path) }.not_to raise_error
      end
    end

    context 'with verbose option' do
      it 'shows verbose output' do
        cli.options = { write: false, verbose: true }

        expect { cli.format(temp_file.path) }.not_to raise_error
      end
    end

    context 'with multiple files' do
      let(:temp_file2) { Tempfile.new(['test2', '.rb']) }

      before do
        temp_file2.write(unformatted_code)
        temp_file2.close
      end

      after do
        temp_file2.unlink
      end

      it 'formats all files' do
        cli.options = { write: true }
        cli.format(temp_file.path, temp_file2.path)

        expect(File.read(temp_file.path)).to eq(formatted_code)
        expect(File.read(temp_file2.path)).to eq(formatted_code)
      end
    end

    context 'with syntax error' do
      let(:invalid_code) { "class Foo\n  def bar\nend" }

      before do
        File.write(temp_file.path, invalid_code)
      end

      it 'handles error gracefully and exits with code 1' do
        cli.options = { write: false }

        expect do
          cli.format(temp_file.path)
        end.to raise_error(SystemExit) do |error|
          expect(error.status).to eq(1)
        end
      end
    end
  end

  describe '#check' do
    let(:temp_file) { Tempfile.new(['test', '.rb']) }

    before do
      temp_file.write(unformatted_code)
      temp_file.close
    end

    after do
      temp_file.unlink
    end

    it 'invokes format with check option' do
      expect(cli).to receive(:invoke).with(:format, [temp_file.path], check: true, write: false)
      cli.check(temp_file.path)
    end
  end

  describe '#init' do
    let(:temp_dir) { Dir.mktmpdir }
    let(:config_file) { File.join(temp_dir, '.rfmt.yml') }
    let(:original_dir) { Dir.pwd }

    before do
      Dir.chdir(temp_dir)
    end

    after do
      Dir.chdir(original_dir)
      FileUtils.rm_rf(temp_dir)
    end

    it 'creates configuration file' do
      cli.options = {}
      cli.init

      expect(File.exist?(config_file)).to be true
    end

    it 'does not overwrite existing config without force' do
      File.write(config_file, 'existing: config')
      cli.options = {}
      cli.init

      expect(File.read(config_file)).to eq('existing: config')
    end

    it 'overwrites existing config with force option' do
      File.write(config_file, 'existing: config')
      cli.options = { force: true }
      cli.init

      content = File.read(config_file)
      expect(content).to include('version')
      expect(content).to include('formatting')
    end
  end

  describe '#config_cmd' do
    it 'displays current configuration' do
      cli.options = {}
      expect { cli.config_cmd }.not_to raise_error
    end
  end

  describe 'integration tests' do
    let(:temp_dir) { Dir.mktmpdir }
    let(:test_file) { File.join(temp_dir, 'test.rb') }
    let(:config_file) { File.join(temp_dir, '.rfmt.yml') }
    let(:original_dir) { Dir.pwd }

    before do
      Dir.chdir(temp_dir)
      File.write(test_file, unformatted_code)
    end

    after do
      Dir.chdir(original_dir)
      FileUtils.rm_rf(temp_dir)
    end

    it 'uses configuration file if present' do
      config_content = <<~YAML
        version: '1.0'
        formatting:
          line_length: 80
          indent_width: 2
          indent_style: spaces
        include:
          - '**/*.rb'
        exclude:
          - 'vendor/**/*'
      YAML
      File.write(config_file, config_content)

      cli.options = { write: true }
      expect { cli.format(test_file) }.not_to raise_error
    end

    it 'formats files matching include patterns' do
      cli.options = { write: true }
      cli.format

      expect(File.read(test_file)).to eq(formatted_code)
    end
  end
end
