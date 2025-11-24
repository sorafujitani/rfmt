# frozen_string_literal: true

require 'yaml'

module Rfmt
  # Configuration management for rfmt
  class Configuration
    class ConfigError < StandardError; end

    DEFAULT_CONFIG = {
      'version' => '1.0',
      'formatting' => {
        'line_length' => 100,
        'indent_width' => 2,
        'indent_style' => 'spaces'
      },
      'include' => ['**/*.rb'],
      'exclude' => ['vendor/**/*', 'tmp/**/*', 'node_modules/**/*']
    }.freeze

    CONFIG_FILES = ['.rfmt.yml', '.rfmt.yaml', 'rfmt.yml', 'rfmt.yaml'].freeze

    attr_reader :config

    def initialize(options = {})
      @config = load_configuration(options)
    end

    # Discover configuration file in current directory
    def self.discover
      config_file = CONFIG_FILES.find { |file| File.exist?(file) }
      config_file ? new(file: config_file) : new
    end

    # Get list of files to format based on include/exclude patterns
    def files_to_format(base_path: '.')
      include_patterns = @config['include']
      exclude_patterns = @config['exclude']

      included_files = include_patterns.flat_map { |pattern| Dir.glob(File.join(base_path, pattern)) }
      excluded_files = exclude_patterns.flat_map { |pattern| Dir.glob(File.join(base_path, pattern)) }

      (included_files - excluded_files).select { |f| File.file?(f) }
    end

    # Get formatting configuration
    def formatting_config
      @config['formatting']
    end

    private

    def load_configuration(options)
      config = deep_dup(DEFAULT_CONFIG)

      # Load from file if specified
      if (file = options[:file] || options['file'])
        file_config = YAML.load_file(file)
        config = deep_merge(config, file_config)
      end

      # Override with options
      options.delete(:file)
      options.delete('file')
      config = deep_merge(config, options) unless options.empty?

      validate_config!(config)
      config
    end

    def validate_config!(config)
      line_length = config.dig('formatting', 'line_length')
      raise ConfigError, 'line_length must be positive' if line_length && line_length <= 0

      indent_width = config.dig('formatting', 'indent_width')
      raise ConfigError, 'indent_width must be positive' if indent_width && indent_width <= 0
    end

    def deep_merge(hash1, hash2)
      hash1.merge(hash2) do |_key, old_val, new_val|
        if old_val.is_a?(Hash) && new_val.is_a?(Hash)
          deep_merge(old_val, new_val)
        else
          new_val
        end
      end
    end

    def deep_dup(hash)
      hash.transform_values do |value|
        value.is_a?(Hash) ? deep_dup(value) : value.dup
      end
    end
  end
end
