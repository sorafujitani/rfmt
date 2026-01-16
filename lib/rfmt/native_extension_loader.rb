# frozen_string_literal: true

module Rfmt
  # Handles loading of native extension across different Ruby versions
  # Ruby 3.3+ places native extensions in version-specific subdirectories
  module NativeExtensionLoader
    class << self
      # Load the native extension, trying multiple possible paths
      # @return [Boolean] true if successfully loaded
      # @raise [LoadError] if the extension cannot be found
      def load_extension
        debug_log "Loading native extension for Ruby #{RUBY_VERSION}"

        possible_paths = build_possible_paths
        debug_log "Trying paths: #{possible_paths.inspect}"

        load_from_paths(possible_paths) || raise(LoadError, build_error_message(possible_paths))
      end

      private

      # Try loading from multiple paths
      # @param paths [Array<String>] paths to try
      # @return [Boolean, nil] true if loaded, nil otherwise
      def load_from_paths(paths)
        paths.each do |path|
          if try_load_extension(path)
            debug_log "Successfully loaded from: #{path}"
            return true
          end
        end
        nil
      end

      # Build list of possible paths for the native extension
      # @return [Array<String>] paths to try, in order of preference
      def build_possible_paths
        paths = []

        # Ruby 3.3+ style: version-specific subdirectory
        paths << version_specific_path if ruby_version >= '3.3'

        # Ruby 3.0-3.2 style: might use version directory
        paths << version_specific_path if ruby_version >= '3.0' && ruby_version < '3.3'

        # Legacy/fallback: direct placement
        paths << File.join(__dir__, 'rfmt')

        # Additional fallback: check for .bundle extension explicitly
        paths << File.join(__dir__, 'rfmt.bundle')

        paths.uniq
      end

      # Get version-specific path
      # @return [String] path with version directory
      def version_specific_path
        File.join(__dir__, ruby_version_dir, 'rfmt')
      end

      # Try to load extension from a specific path
      # @param path [String] path to try
      # @return [Boolean] true if successful, false otherwise
      def try_load_extension(path)
        require path
        true
      rescue LoadError => e
        debug_log "Failed to load from #{path}: #{e.message}"
        false
      end

      # Get Ruby version for comparison
      # @return [String] Ruby version string
      def ruby_version
        RUBY_VERSION
      end

      # Get Ruby version directory name (e.g., "3.3" for Ruby 3.3.0)
      # @return [String] version directory name
      def ruby_version_dir
        RUBY_VERSION.split('.')[0..1].join('.')
      end

      # Build detailed error message when extension cannot be loaded
      # @param tried_paths [Array<String>] paths that were tried
      # @return [String] error message
      def build_error_message(tried_paths)
        [
          error_header,
          format_tried_paths(tried_paths),
          error_explanation,
          workaround_instructions
        ].join("\n")
      end

      # Error message header
      # @return [String] header text
      def error_header
        "Unable to load rfmt native extension for Ruby #{RUBY_VERSION}.\n"
      end

      # Format list of tried paths
      # @param paths [Array<String>] paths that were tried
      # @return [String] formatted path list
      def format_tried_paths(paths)
        "Tried the following paths:\n#{paths.map { |p| "  - #{p}" }.join("\n")}\n"
      end

      # Error explanation text
      # @return [String] explanation
      def error_explanation
        "This might be a packaging issue with the gem for your Ruby version.\n"
      end

      # Workaround instructions
      # @return [String] instructions
      def workaround_instructions
        <<~MSG.chomp
          Workaround:
          1. Check if rfmt.bundle exists in: #{__dir__}/
          2. If it's in a subdirectory, create a symlink:
             cd #{__dir__}
             ln -sf <subdirectory>/rfmt.bundle rfmt.bundle

          Please report this issue at: https://github.com/fs0414/rfmt/issues
        MSG
      end

      # Log debug information if RFMT_DEBUG is set
      # @param message [String] message to log
      def debug_log(message)
        return unless ENV['RFMT_DEBUG']

        warn "[RFMT::NativeExtensionLoader] #{message}"
      end
    end
  end
end
