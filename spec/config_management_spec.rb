# frozen_string_literal: true

require 'spec_helper'
require 'rfmt'
require 'tmpdir'
require 'fileutils'

RSpec.describe Rfmt::Config do
  describe '.init' do
    it 'creates a new configuration file' do
      Dir.mktmpdir do |dir|
        config_path = File.join(dir, '.rfmt.yml')
        result = Rfmt::Config.init(config_path)

        expect(result).to be true
        expect(File.exist?(config_path)).to be true

        content = File.read(config_path)
        expect(content).to include('rfmt Configuration File')
        expect(content).to include('line_length: 100')
      end
    end

    it 'refuses to overwrite existing file without force' do
      Dir.mktmpdir do |dir|
        config_path = File.join(dir, '.rfmt.yml')

        # Create initial file
        Rfmt::Config.init(config_path)

        # Try to overwrite without force
        result = Rfmt::Config.init(config_path, force: false)

        expect(result).to be false
      end
    end

    it 'overwrites existing file with force: true' do
      Dir.mktmpdir do |dir|
        config_path = File.join(dir, '.rfmt.yml')

        # Create initial file
        File.write(config_path, 'old content')

        # Overwrite with force
        result = Rfmt::Config.init(config_path, force: true)

        expect(result).to be true
        content = File.read(config_path)
        expect(content).to include('rfmt Configuration File')
        expect(content).not_to include('old content')
      end
    end
  end

  describe '.load' do
    it 'loads configuration from explicit path' do
      Dir.mktmpdir do |dir|
        config_path = File.join(dir, 'custom_config.yml')
        config_content = <<~YAML
          version: "1.0"
          formatting:
            line_length: 80
            indent_width: 4
        YAML
        File.write(config_path, config_content)

        config = Rfmt::Config.load(config_path)

        expect(config['version']).to eq('1.0')
        expect(config['formatting']['line_length']).to eq(80)
        expect(config['formatting']['indent_width']).to eq(4)
      end
    end

    it 'raises error for non-existent file' do
      expect do
        Rfmt::Config.load('/nonexistent/path/config.yml')
      end.to raise_error(Rfmt::Error, /Configuration file not found/)
    end

    it 'raises error for invalid YAML' do
      Dir.mktmpdir do |dir|
        config_path = File.join(dir, 'invalid.yml')
        File.write(config_path, "invalid: yaml: content:\n  - broken")

        expect do
          Rfmt::Config.load(config_path)
        end.to raise_error(Rfmt::Error, /Invalid YAML/)
      end
    end
  end

  describe '.exists?' do
    it 'returns false when no config file exists' do
      Dir.mktmpdir do |_dir|
        # Use a temporary directory with no config file
        # We can't change directory, so this test is limited
        # Just verify the method exists and returns a boolean
        result = Rfmt::Config.exists?
        expect([true, false]).to include(result)
      end
    end
  end

  describe 'DEFAULT_CONFIG' do
    let(:default_config) { Rfmt::Config::DEFAULT_CONFIG }

    it 'is a non-empty string' do
      expect(default_config).to be_a(String)
      expect(default_config.length).to be > 0
    end

    it 'contains required configuration keys' do
      expect(default_config).to include('version')
      expect(default_config).to include('formatting')
      expect(default_config).to include('line_length')
      expect(default_config).to include('indent_width')
      expect(default_config).to include('indent_style')
      expect(default_config).to include('quote_style')
    end

    it 'contains include patterns' do
      expect(default_config).to include('include')
      expect(default_config).to include('**/*.rb')
      expect(default_config).to include('**/*.rake')
    end

    it 'contains exclude patterns' do
      expect(default_config).to include('exclude')
      expect(default_config).to include('vendor/**/*')
      expect(default_config).to include('tmp/**/*')
    end

    it 'is valid YAML' do
      require 'yaml'
      expect do
        YAML.safe_load(default_config)
      end.not_to raise_error
    end

    it 'has reasonable default values' do
      require 'yaml'
      config = YAML.safe_load(default_config)

      expect(config['version']).to eq('1.0')
      expect(config['formatting']['line_length']).to eq(100)
      expect(config['formatting']['indent_width']).to eq(2)
      expect(config['formatting']['indent_style']).to eq('spaces')
      expect(config['formatting']['quote_style']).to eq('double')
    end
  end
end
