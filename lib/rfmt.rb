# frozen_string_literal: true

require_relative "rfmt/version"
require_relative "rfmt/rfmt"
require_relative "rfmt/prism_bridge"

module Rfmt
  class Error < StandardError; end
  class RfmtError < Error; end  # Errors from Rust side
  class ValidationError < RfmtError; end  # AST validation errors

  # Format Ruby source code
  # @param source [String] Ruby source code to format
  # @return [String] Formatted Ruby code
  def self.format(source)
    # Step 1: Parse with Prism (Ruby side)
    prism_json = PrismBridge.parse(source)

    # Step 2: Format in Rust
    # Pass both source and AST to enable source extraction fallback
    formatted = format_code(source, prism_json)

    formatted
  rescue PrismBridge::ParseError => e
    # Re-raise with more context
    raise Error, "Failed to parse Ruby code: #{e.message}"
  rescue RfmtError => e
    # Rust side errors are re-raised as-is to preserve error details
    raise
  rescue => e
    raise Error, "Unexpected error during formatting: #{e.class}: #{e.message}"
  end

  # Format a Ruby file
  # @param path [String] Path to Ruby file
  # @return [String] Formatted Ruby code
  def self.format_file(path)
    source = File.read(path)
    format(source)
  rescue Errno::ENOENT
    raise Error, "File not found: #{path}"
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
    prism_json = PrismBridge.parse(source)
    parse_to_json(prism_json)
  end
end
