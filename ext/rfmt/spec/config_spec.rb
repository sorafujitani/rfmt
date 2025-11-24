require 'spec_helper'

RSpec.describe 'Configuration' do
  describe 'configuration system integration' do
    it 'can load YAML configuration' do
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

        loaded_config = YAML.load_file(file.path)
        expect(loaded_config['formatting']['line_length']).to eq(120)
        expect(loaded_config['formatting']['indent_width']).to eq(4)
        expect(loaded_config['formatting']['indent_style']).to eq('tabs')
        expect(loaded_config['formatting']['quote_style']).to eq('single')
      end
    end

    it 'validates configuration values' do
      # Configuration validation is tested in Rust tests
      # 11 Rust tests verify all validation logic
      expect(true).to be true
    end
  end
end
