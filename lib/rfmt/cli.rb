# frozen_string_literal: true

require 'thor'

# Check for verbose flag before loading rfmt to set debug mode early
ENV['RFMT_DEBUG'] = '1' if ARGV.include?('-v') || ARGV.include?('--verbose')

require 'rfmt'
require 'rfmt/configuration'
require 'rfmt/cache'

module Rfmt
  # Cache management commands
  class CacheCommands < Thor
    desc 'clear', 'Clear all cache data'
    option :cache_dir, type: :string, desc: 'Cache directory (default: ~/.cache/rfmt)'
    def clear
      cache_opts = options[:cache_dir] ? { cache_dir: options[:cache_dir] } : {}
      cache = Cache.new(**cache_opts)
      cache.clear
      say 'Cache cleared', :green
    end

    desc 'stats', 'Show cache statistics'
    option :cache_dir, type: :string, desc: 'Cache directory (default: ~/.cache/rfmt)'
    def stats
      cache_opts = options[:cache_dir] ? { cache_dir: options[:cache_dir] } : {}
      cache = Cache.new(**cache_opts)
      stats = cache.stats
      say "Cache directory: #{stats[:cache_dir]}", :blue
      say "Total files in cache: #{stats[:total_files]}", :blue
      say "Cache size: #{(stats[:cache_size_bytes] / 1024.0).round(2)} KB", :blue
    end

    desc 'prune', 'Remove cache entries for files that no longer exist'
    option :cache_dir, type: :string, desc: 'Cache directory (default: ~/.cache/rfmt)'
    def prune
      cache_opts = options[:cache_dir] ? { cache_dir: options[:cache_dir] } : {}
      cache = Cache.new(**cache_opts)
      pruned = cache.prune
      say "Pruned #{pruned} stale cache entries", :green
    end
  end

  # Command Line Interface for rfmt
  class CLI < Thor
    class_option :config, type: :string, desc: 'Path to configuration file'
    class_option :verbose, type: :boolean, aliases: '-v', desc: 'Verbose output'

    default_command :format

    desc 'format [FILES]', 'Format Ruby files (default command)'
    option :write, type: :boolean, default: true, desc: 'Write formatted output'
    option :check, type: :boolean, desc: "Check if files are formatted (don't write)"
    option :diff, type: :boolean, desc: 'Show diff of changes'
    option :diff_format, type: :string, default: 'unified', desc: 'Diff format: unified, side_by_side, or color'
    option :parallel, type: :boolean, desc: 'Use parallel processing (auto-disabled for <20 files)'
    option :jobs, type: :numeric, desc: 'Number of parallel jobs (default: CPU count)'
    option :cache, type: :boolean, default: true, desc: 'Use cache to skip unchanged files'
    option :cache_dir, type: :string, desc: 'Cache directory (default: ~/.cache/rfmt)'
    def format(*files)
      config = load_config
      files = files.empty? ? config.files_to_format : files.flatten

      if files.empty?
        say 'No files to format', :yellow
        return
      end

      # Initialize cache
      cache = if options[:cache]
                cache_opts = options[:cache_dir] ? { cache_dir: options[:cache_dir] } : {}
                Cache.new(**cache_opts)
              end

      # Filter files using cache
      if cache
        original_count = files.size
        files = files.select { |file| cache.needs_formatting?(file) }
        skipped = original_count - files.size
        say "ℹ Skipped #{skipped} unchanged file(s) (cached)", :cyan if skipped.positive? && options[:verbose]
      end

      if files.empty?
        say '✓ All files are already formatted (cached)', :green
        return
      end

      # Show progress message
      if files.size == 1
        say "Processing #{files.first}...", :blue
      else
        say "Processing #{files.size} file(s)...", :blue
      end

      use_parallel = should_use_parallel?(files)

      if options[:verbose] && files.size > 1
        mode = use_parallel ? "parallel (#{options[:jobs] || 'auto'} jobs)" : 'sequential'
        say "Using #{mode} processing for #{files.size} files", :blue
      end

      results = if use_parallel
                  format_files_parallel(files)
                else
                  format_files_sequential(files)
                end
      handle_results(results, cache)
    end

    desc 'check [FILES]', 'Check if files need formatting'
    def check(*files)
      invoke :format, files, check: true, write: false
    end

    desc 'version', 'Show version'
    def version
      say "rfmt #{Rfmt::VERSION}"
      say "Rust extension: #{Rfmt.rust_version}"
    end

    desc 'config', 'Show current configuration'
    def config_cmd
      config = load_config
      require 'json'
      say JSON.pretty_generate(config.config)
    end

    desc 'cache SUBCOMMAND', 'Manage cache'
    subcommand 'cache', CacheCommands

    desc 'init', 'Initialize rfmt configuration'
    option :force, type: :boolean, desc: 'Overwrite existing configuration'
    option :path, type: :string, default: '.rfmt.yml', desc: 'Configuration file path'
    def init
      config_file = options[:path] || '.rfmt.yml'

      # Use Rfmt::Config module for consistent behavior
      result = Rfmt::Config.init(config_file, force: options[:force] || false)

      if result
        say "Created #{config_file}", :green
      else
        say "Configuration file already exists at #{config_file}. Use --force to overwrite.", :yellow
      end
    end

    private

    # Intelligently decide whether to use parallel processing
    def should_use_parallel?(files)
      return false if files.size <= 1

      # Check if parallel option was explicitly set via command line
      # Thor sets options[:parallel] to true/false for --parallel/--no-parallel
      # and nil when not specified
      return options[:parallel] unless options[:parallel].nil?

      # Auto decision based on workload characteristics
      # Calculate total size for better decision
      total_size = files.sum do |f|
        File.size(f)
      rescue StandardError
        0
      end
      avg_size = total_size / files.size.to_f

      # Decision matrix:
      # - Less than 20 files: sequential (overhead > benefit)
      # - 20-50 files with small size (<10KB avg): sequential
      # - 20-50 files with large size (>10KB avg): parallel
      # - More than 50 files: always parallel

      if files.size < 20
        false
      elsif files.size < 50
        avg_size > 10_000 # 10KB threshold
      else
        true
      end
    end

    def load_config
      if options[:config]
        Configuration.new(file: options[:config])
      else
        Configuration.discover
      end
    end

    def format_files_sequential(files)
      files.map do |file|
        format_single_file(file)
      end
    end

    def format_files_parallel(files)
      require 'parallel'

      # Determine number of processes to use
      process_count = options[:jobs] || Parallel.processor_count

      say "Processing #{files.size} files with #{process_count} parallel jobs...", :blue if options[:verbose]

      Parallel.map(files, in_processes: process_count) do |file|
        format_single_file(file)
      end
    end

    def format_single_file(file)
      start_time = Time.now
      source = File.read(file)

      formatted = Rfmt.format(source)
      changed = source != formatted

      {
        file: file,
        changed: changed,
        original: source,
        formatted: formatted,
        duration: Time.now - start_time,
        error: nil
      }
    rescue StandardError => e
      {
        file: file,
        error: e.message,
        duration: Time.now - start_time
      }
    end

    def handle_results(results, cache = nil)
      failed_count = 0
      changed_count = 0
      error_count = 0

      results.each do |result|
        if result[:error]
          say "Error in #{result[:file]}: #{result[:error]}", :red
          error_count += 1
          next
        end

        if result[:changed]
          changed_count += 1

          if options[:check]
            say "#{result[:file]} needs formatting", :yellow
            failed_count += 1
            show_diff(result[:file], result[:original], result[:formatted]) if options[:diff]
          elsif options[:diff]
            show_diff(result[:file], result[:original], result[:formatted])
          elsif options[:write]
            File.write(result[:file], result[:formatted])
            # Always show formatted files (not just in verbose mode)
            say "✓ Formatted #{result[:file]}", :green

            # Update cache after successful write
            cache&.mark_formatted(result[:file])
          else
            puts result[:formatted]
          end
        else
          # Show already formatted files in non-check mode
          say "✓ #{result[:file]} already formatted", :cyan unless options[:check]

          # Update cache even if no changes (file was checked)
          cache&.mark_formatted(result[:file])
        end
      end

      # Save cache to disk
      cache&.save

      # Summary - always show a summary message
      if error_count.positive?
        say "\n✗ Failed: #{error_count} error(s) occurred", :red
      elsif options[:check] && failed_count.positive?
        say "\n✗ Check failed: #{failed_count} file(s) need formatting", :yellow
      elsif changed_count.positive?
        # Success message with appropriate details
        say "\n✓ Success! Formatted #{changed_count} file(s)", :green
      elsif results.size == 1
        say "\n✓ Success! File is already formatted", :green
      else
        say "\n✓ Success! All #{results.size} files are already formatted", :green
      end

      # Detailed summary in verbose mode
      if options[:verbose]
        say "Total: #{results.size} file(s) processed", :blue
        say "Changed: #{changed_count} file(s)", :yellow if changed_count.positive?
      end

      exit(1) if (options[:check] && failed_count.positive?) || error_count.positive?
    end

    def show_diff(file, original, formatted)
      require 'diffy'

      say "\n#{'=' * 80}", :blue
      say "Diff for #{file}:", :yellow
      say '=' * 80, :blue

      case options[:diff_format]
      when 'unified'
        diff = Diffy::Diff.new(original, formatted, context: 3)
        puts diff.to_s(:color)
      when 'side_by_side'
        diff = Diffy::Diff.new(original, formatted, context: 3)
        # Side-by-side is not well supported in terminal, use unified with more context
        puts diff.to_s(:color)
      when 'color'
        show_colored_line_diff(original, formatted)
      else
        diff = Diffy::Diff.new(original, formatted, context: 3)
        puts diff.to_s(:color)
      end

      say "#{'=' * 80}\n", :blue
    end

    def show_colored_line_diff(original, formatted)
      require 'diff/lcs'

      original_lines = original.split("\n")
      formatted_lines = formatted.split("\n")

      diffs = Diff::LCS.sdiff(original_lines, formatted_lines)

      diffs.each_with_index do |diff, idx|
        line_num = idx + 1
        case diff.action
        when '-'
          say "#{line_num}: - #{diff.old_element}", :red
        when '+'
          say "#{line_num}: + #{diff.new_element}", :green
        when '='
          say "#{line_num}:   #{diff.old_element}", :white
        when '!'
          say "#{line_num}: - #{diff.old_element}", :red
          say "#{line_num}: + #{diff.new_element}", :green
        end
      end
    end
  end
end
