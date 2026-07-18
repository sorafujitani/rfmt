# frozen_string_literal: true

require_relative 'rfmt/version'
require_relative 'rfmt/native_extension_loader'

# Load native extension with version-aware loader
Rfmt::NativeExtensionLoader.load_extension

module Rfmt
  class Error < StandardError; end
  # Errors from Rust side
  class RfmtError < Error; end
  # AST validation errors
  class ValidationError < RfmtError; end

  # Rust reports errors as plain StandardError with a [Rfmt::<kind>] prefix;
  # these two kinds map onto the public exception classes.
  NATIVE_PARSE_ERROR_PREFIX = '[Rfmt::ParseError] '
  NATIVE_VALIDATION_ERROR_PREFIX = '[Rfmt::ValidationError] '
  NATIVE_CONFIG_ERROR_PREFIX = '[Rfmt::ConfigError] '
  private_constant :NATIVE_PARSE_ERROR_PREFIX, :NATIVE_VALIDATION_ERROR_PREFIX,
                   :NATIVE_CONFIG_ERROR_PREFIX

  # Format Ruby source code
  # Parsing, config resolution, and output validation all happen natively in Rust
  # @param source [String] Ruby source code to format
  # @param config_path [String, nil] Explicit config file path; nil discovers
  #   rfmt.yml/.rfmt.yml from the current directory upward (cached per process)
  # @return [String] Formatted Ruby code
  def self.format(source, config_path: nil)
    if config_path
      format_code_with_config(source, config_path.to_s)
    else
      format_code(source)
    end
  rescue StandardError => e
    raise wrap_native_error(e)
  end

  def self.wrap_native_error(error)
    message = error.message
    if message.start_with?(NATIVE_PARSE_ERROR_PREFIX)
      Error.new("Failed to parse Ruby code: #{message.delete_prefix(NATIVE_PARSE_ERROR_PREFIX)}")
    elsif message.start_with?(NATIVE_VALIDATION_ERROR_PREFIX)
      ValidationError.new(message.delete_prefix(NATIVE_VALIDATION_ERROR_PREFIX))
    elsif message.start_with?(NATIVE_CONFIG_ERROR_PREFIX)
      Error.new("Configuration error: #{message.delete_prefix(NATIVE_CONFIG_ERROR_PREFIX)}")
    else
      Error.new("Unexpected error during formatting: #{error.class}: #{message}")
    end
  end
  private_class_method :wrap_native_error

  # Format a Ruby file
  # @param path [String] Path to Ruby file
  # @return [String] Formatted Ruby code
  def self.format_file(path)
    source = File.read(path)
    format(source)
  rescue Errno::ENOENT
    raise Error, "File not found: #{path}"
  end

  # Effective configuration as the Rust formatter resolves it
  # @param config_path [String, nil] Explicit config file path; nil discovers
  # @return [String] YAML dump of the resolved configuration
  def self.resolved_config(config_path: nil)
    resolved_config_yaml(config_path&.to_s)
  rescue StandardError => e
    raise wrap_native_error(e)
  end

  # Get version information
  # @return [String] Version string including Ruby and Rust versions
  def self.version_info
    "Ruby: #{VERSION}, Rust: #{rust_version}"
  end

  # Parse Ruby code to AST (for debugging)
  # @param source [String] Ruby source code
  # @return [String] AST representation
  def self.parse(source)
    parse_to_json(source)
  rescue StandardError => e
    raise wrap_native_error(e)
  end

  # Configuration management
  module Config
    # Default configuration template
    DEFAULT_CONFIG = <<~YAML
      # rfmt Configuration File
      # This file controls how rfmt formats your Ruby code.
      # See https://github.com/fs0414/rfmt for full documentation.

      version: "1.0"

      # Formatting options
      formatting:
        # Maximum line length before wrapping (40-500)
        line_length: 100

        # Number of spaces or tabs per indentation level (1-8)
        indent_width: 2

        # Use "spaces" or "tabs" for indentation
        indent_style: "spaces"

        # Quote style for strings: "double", "single", or "consistent"
        quote_style: "double"

      # Files to include in formatting (glob patterns)
      include:
        - "**/*.rb"
        - "**/*.rake"
        - "**/Rakefile"
        - "**/Gemfile"

      # Files to exclude from formatting (glob patterns)
      exclude:
        - "vendor/**/*"
        - "tmp/**/*"
        - "node_modules/**/*"
        - "db/schema.rb"
    YAML

    # Generate a default configuration file
    # @param path [String] Path where to create the config file (default: .rfmt.yml)
    # @param force [Boolean] Overwrite existing file if true
    # @return [Boolean] true if file was created, false if already exists
    def self.init(path = '.rfmt.yml', force: false)
      return false if File.exist?(path) && !force

      File.write(path, DEFAULT_CONFIG)
      true
    end

    # Find configuration file in current or parent directories
    # @return [String, nil] Path to config file or nil if not found
    def self.find
      current_dir = Dir.pwd

      loop do
        # Same search order as the Rust side (config/mod.rs CONFIG_FILE_NAMES)
        ['rfmt.yml', 'rfmt.yaml', '.rfmt.yml', '.rfmt.yaml'].each do |filename|
          config_path = File.join(current_dir, filename)
          return config_path if File.exist?(config_path)
        end

        parent = File.dirname(current_dir)
        break if parent == current_dir # Reached root

        current_dir = parent
      end

      # Check user home directory
      home_dir = begin
        Dir.home
      rescue StandardError
        nil
      end
      if home_dir
        ['rfmt.yml', 'rfmt.yaml', '.rfmt.yml', '.rfmt.yaml'].each do |filename|
          config_path = File.join(home_dir, filename)
          return config_path if File.exist?(config_path)
        end
      end

      nil
    end

    # Check if configuration file exists
    # @return [Boolean] true if config file exists
    def self.exists?
      !find.nil?
    end

    # Load and validate configuration file
    # @param path [String, nil] Path to config file (default: auto-detect)
    # @return [Hash] Loaded configuration
    def self.load(path = nil)
      require 'yaml'

      config_path = path || find

      unless config_path
        warn 'No configuration file found, using defaults'
        return {}
      end

      YAML.load_file(config_path)
    rescue Errno::ENOENT
      raise Error, "Configuration file not found: #{config_path}"
    rescue Psych::SyntaxError => e
      raise Error, "Invalid YAML in configuration file: #{e.message}"
    end
  end
end
