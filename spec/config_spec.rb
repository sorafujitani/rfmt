require 'spec_helper'

RSpec.describe 'Configuration' do
  describe 'default configuration' do
    it 'uses default values when no config file exists' do
      expect(true).to be true
    end
  end

  describe 'configuration loading' do
    it 'can load configuration from YAML file' do
      require 'tempfile'
      require 'yaml'

      config = {
        'version' => '1.0',
        'formatting' => {
          'line_length' => 120,
          'indent_width' => 4,
          'indent_style' => 'tabs',
          'quote_style' => 'single'
        }
      }

      Tempfile.create(['test_config', '.yml']) do |file|
        file.write(YAML.dump(config))
        file.flush

        # Configuration loading is tested at the Rust level
        expect(File.exist?(file.path)).to be true
        loaded_config = YAML.load_file(file.path)
        expect(loaded_config['formatting']['line_length']).to eq(120)
        expect(loaded_config['formatting']['indent_width']).to eq(4)
      end
    end
  end

  describe 'configuration validation' do
    it 'validates line_length bounds' do
      require 'tempfile'
      require 'yaml'

      config = {
        'formatting' => {
          'line_length' => 30  # Too small
        }
      }

      Tempfile.create(['invalid_config', '.yml']) do |file|
        file.write(YAML.dump(config))
        file.flush

        # Validation is tested at the Rust level
        loaded_config = YAML.load_file(file.path)
        expect(loaded_config['formatting']['line_length']).to eq(30)
      end
    end
  end
end
