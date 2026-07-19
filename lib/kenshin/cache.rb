# frozen_string_literal: true

require 'json'
require 'fileutils'

module Kenshin
  # Cache system for formatted files
  # Uses mtime (modification time) to determine if formatting is needed
  class Cache
    class CacheError < StandardError; end

    DEFAULT_CACHE_DIR = File.expand_path('~/.cache/kenshin').freeze
    CACHE_VERSION = '1'

    attr_reader :cache_dir

    def initialize(cache_dir: DEFAULT_CACHE_DIR)
      @cache_dir = cache_dir
      @cache_data = {}
      ensure_cache_dir
      load_cache
    end

    # Check if file needs formatting
    # Returns true if file mtime has changed or not in cache
    def needs_formatting?(file_path)
      return true unless File.exist?(file_path)

      current_mtime = File.mtime(file_path).to_i
      cached_mtime = @cache_data.dig(file_path, 'mtime')

      current_mtime != cached_mtime
    end

    # Mark file as formatted with current mtime
    def mark_formatted(file_path)
      return unless File.exist?(file_path)

      @cache_data[file_path] = {
        'mtime' => File.mtime(file_path).to_i,
        'formatted_at' => Time.now.to_i,
        'version' => CACHE_VERSION
      }
    end

    # Save cache to disk
    def save
      cache_file = File.join(@cache_dir, 'cache.json')
      File.write(cache_file, JSON.pretty_generate(@cache_data))
    end

    # Clear all cache data
    def clear
      @cache_data = {}
      save
    end

    # Remove cache for specific file
    def invalidate(file_path)
      @cache_data.delete(file_path)
    end

    # Get cache statistics
    def stats
      {
        total_files: @cache_data.size,
        cache_dir: @cache_dir,
        cache_size_bytes: cache_size
      }
    end

    # Prune old cache entries (files that no longer exist)
    def prune
      before_count = @cache_data.size
      @cache_data.delete_if { |file_path, _| !File.exist?(file_path) }
      after_count = @cache_data.size
      pruned = before_count - after_count

      save if pruned.positive?
      pruned
    end

    private

    def ensure_cache_dir
      FileUtils.mkdir_p(@cache_dir)
    rescue StandardError => e
      raise CacheError, "Failed to create cache directory: #{e.message}"
    end

    def load_cache
      cache_file = File.join(@cache_dir, 'cache.json')
      return unless File.exist?(cache_file)

      content = File.read(cache_file)
      @cache_data = JSON.parse(content)
    rescue JSON::ParserError => e
      warn "Warning: Failed to parse cache file, starting with empty cache: #{e.message}"
      @cache_data = {}
    rescue StandardError => e
      warn "Warning: Failed to load cache, starting with empty cache: #{e.message}"
      @cache_data = {}
    end

    def cache_size
      cache_file = File.join(@cache_dir, 'cache.json')
      return 0 unless File.exist?(cache_file)

      File.size(cache_file)
    end
  end
end
