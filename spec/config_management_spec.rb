require 'spec_helper'
require 'tempfile'
require 'tmpdir'

RSpec.describe Rfmt::Config do
  describe '.init' do
    it 'creates a new configuration file' do
      Dir.mktmpdir do |dir|
        config_path = File.join(dir, '.rfmt.yml')

        result = Rfmt::Config.init(config_path)

        expect(result).to be true
        expect(File.exist?(config_path)).to be true

        content = File.read(config_path)
        expect(content).to include('version: "1.0"')
        expect(content).to include('line_length: 100')
        expect(content).to include('indent_width: 2')
      end
    end

    it 'does not overwrite existing file without force' do
      Dir.mktmpdir do |dir|
        config_path = File.join(dir, '.rfmt.yml')
        File.write(config_path, 'existing content')

        result = Rfmt::Config.init(config_path)

        expect(result).to be false
        expect(File.read(config_path)).to eq('existing content')
      end
    end

    it 'overwrites existing file with force: true' do
      Dir.mktmpdir do |dir|
        config_path = File.join(dir, '.rfmt.yml')
        File.write(config_path, 'existing content')

        result = Rfmt::Config.init(config_path, force: true)

        expect(result).to be true
        content = File.read(config_path)
        expect(content).to include('version: "1.0"')
        expect(content).not_to include('existing content')
      end
    end
  end

  describe '.find' do
    it 'finds config file in current directory' do
      original_dir = Dir.pwd
      Dir.mktmpdir do |dir|
        config_path = File.join(dir, '.rfmt.yml')
        File.write(config_path, 'test')

        begin
          Dir.chdir(dir)
          found = Rfmt::Config.find
          expect(File.realpath(found)).to eq(File.realpath(config_path))
        ensure
          Dir.chdir(original_dir) if Dir.exist?(original_dir)
        end
      end
    end

    it 'finds config file in parent directory' do
      Dir.mktmpdir do |dir|
        config_path = File.join(dir, '.rfmt.yml')
        File.write(config_path, 'test')

        subdir = File.join(dir, 'sub')
        Dir.mkdir(subdir)

        original_dir = Dir.pwd
        begin
          Dir.chdir(subdir)
          found = Rfmt::Config.find
          expect(File.realpath(found)).to eq(File.realpath(config_path))
        ensure
          Dir.chdir(original_dir)
        end
      end
    end

    it 'returns nil when no config file exists' do
      Dir.mktmpdir do |dir|
        original_dir = Dir.pwd
        begin
          Dir.chdir(dir)
          found = Rfmt::Config.find
          expect(found).to be_nil
        ensure
          Dir.chdir(original_dir)
        end
      end
    end

    it 'prefers .rfmt.yml over .rfmt.yaml' do
      Dir.mktmpdir do |dir|
        yml_path = File.join(dir, '.rfmt.yml')
        yaml_path = File.join(dir, '.rfmt.yaml')

        File.write(yml_path, 'yml')
        File.write(yaml_path, 'yaml')

        original_dir = Dir.pwd
        begin
          Dir.chdir(dir)
          found = Rfmt::Config.find
          expect(File.realpath(found)).to eq(File.realpath(yml_path))
        ensure
          Dir.chdir(original_dir)
        end
      end
    end
  end

  describe '.exists?' do
    it 'returns true when config file exists' do
      Dir.mktmpdir do |dir|
        config_path = File.join(dir, '.rfmt.yml')
        File.write(config_path, 'test')

        original_dir = Dir.pwd
        begin
          Dir.chdir(dir)
          expect(Rfmt::Config.exists?).to be true
        ensure
          Dir.chdir(original_dir)
        end
      end
    end

    it 'returns false when no config file exists' do
      Dir.mktmpdir do |dir|
        original_dir = Dir.pwd
        begin
          Dir.chdir(dir)
          expect(Rfmt::Config.exists?).to be false
        ensure
          Dir.chdir(original_dir)
        end
      end
    end
  end

  describe '.load' do
    it 'loads valid YAML configuration' do
      Dir.mktmpdir do |dir|
        config_path = File.join(dir, '.rfmt.yml')
        config_content = <<~YAML
          version: "1.0"
          formatting:
            line_length: 120
            indent_width: 4
        YAML
        File.write(config_path, config_content)

        original_dir = Dir.pwd
        begin
          Dir.chdir(dir)
          config = Rfmt::Config.load
          expect(config['version']).to eq('1.0')
          expect(config['formatting']['line_length']).to eq(120)
          expect(config['formatting']['indent_width']).to eq(4)
        ensure
          Dir.chdir(original_dir)
        end
      end
    end

    it 'returns empty hash when no config file found' do
      Dir.mktmpdir do |dir|
        original_dir = Dir.pwd
        begin
          Dir.chdir(dir)
          config = Rfmt::Config.load
          expect(config).to eq({})
        ensure
          Dir.chdir(original_dir)
        end
      end
    end

    it 'raises error for invalid YAML' do
      Dir.mktmpdir do |dir|
        config_path = File.join(dir, '.rfmt.yml')
        File.write(config_path, "invalid: yaml: content:")

        original_dir = Dir.pwd
        begin
          Dir.chdir(dir)
          expect {
            Rfmt::Config.load
          }.to raise_error(Rfmt::Error, /Invalid YAML/)
        ensure
          Dir.chdir(original_dir)
        end
      end
    end

    it 'can load from explicit path' do
      Dir.mktmpdir do |dir|
        config_path = File.join(dir, 'custom_config.yml')
        config_content = <<~YAML
          version: "1.0"
          formatting:
            line_length: 80
        YAML
        File.write(config_path, config_content)

        config = Rfmt::Config.load(config_path)
        expect(config['formatting']['line_length']).to eq(80)
      end
    end
  end

  describe 'DEFAULT_CONFIG' do
    it 'contains all required fields' do
      config = Rfmt::Config::DEFAULT_CONFIG

      expect(config).to include('version')
      expect(config).to include('formatting')
      expect(config).to include('line_length')
      expect(config).to include('indent_width')
      expect(config).to include('indent_style')
      expect(config).to include('quote_style')
      expect(config).to include('include')
      expect(config).to include('exclude')
    end

    it 'has valid YAML syntax' do
      require 'yaml'

      expect {
        YAML.load(Rfmt::Config::DEFAULT_CONFIG)
      }.not_to raise_error
    end

    it 'includes helpful comments' do
      config = Rfmt::Config::DEFAULT_CONFIG

      expect(config).to include('# rfmt Configuration File')
      expect(config).to include('Maximum line length')
      expect(config).to include('glob patterns')
    end
  end
end
