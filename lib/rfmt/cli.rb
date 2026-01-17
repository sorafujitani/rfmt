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
    # Constants
    PROGRESS_THRESHOLD = 20  # Show progress for file counts >= this
    PROGRESS_INTERVAL = 10   # Update progress every N files

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
    option :quiet, type: :boolean, aliases: '-q', desc: 'Minimal output (errors and summary only)'
    def format(*files)
      config = load_config
      files = files.empty? ? config.files_to_format : files.flatten

      if files.empty?
        say 'No files to format', :yellow
        return
      end

      # Initialize and use cache if enabled
      cache = initialize_cache_if_enabled
      files = filter_files_with_cache(files, cache)

      if files.empty?
        say '✓ All files are already formatted (cached)', :cyan
        return
      end

      # Show progress message (unless in quiet mode)
      unless options[:quiet]
        if files.size == 1
          say "Processing #{files.first}...", :blue
        else
          say "Processing #{files.size} file(s)...", :blue
        end
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

    def initialize_cache_if_enabled
      return nil unless options[:cache]

      cache_opts = options[:cache_dir] ? { cache_dir: options[:cache_dir] } : {}
      Cache.new(**cache_opts)
    end

    def filter_files_with_cache(files, cache)
      return files unless cache

      original_count = files.size
      filtered = files.select { |file| cache.needs_formatting?(file) }

      log_cache_skip(original_count - filtered.size)
      filtered
    end

    def log_cache_skip(skipped_count)
      return unless skipped_count.positive? && options[:verbose]

      say "ℹ Skipped #{skipped_count} unchanged file(s) (cached)", :cyan
    end

    def format_files_sequential(files)
      show_progress = should_show_progress?(files)

      files.map.with_index do |file, index|
        display_progress(index, files.size) if show_progress && (index % PROGRESS_INTERVAL).zero?
        format_single_file(file)
      end
    end

    def should_show_progress?(files)
      !options[:quiet] && files.size >= PROGRESS_THRESHOLD
    end

    def display_progress(index, total)
      percentage = ((index.to_f / total) * 100).round
      say "[#{index}/#{total}] #{percentage}% complete...", :blue
    end

    def format_files_parallel(files)
      require 'parallel'

      process_count = determine_process_count
      log_parallel_processing(files.size, process_count)

      Parallel.map(files, in_processes: process_count) do |file|
        format_single_file(file)
      end
    end

    def determine_process_count
      options[:jobs] || Parallel.processor_count
    end

    def log_parallel_processing(file_count, process_count)
      return unless options[:verbose]

      say "Processing #{file_count} files with #{process_count} parallel jobs...", :blue
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
      stats = process_results(results, cache)
      stats[:total_duration] = results.sum { |r| r[:duration] || 0 }
      cache&.save
      display_summary(stats, results.size)
      exit(1) if should_exit_with_error?(stats)
    end

    def process_results(results, cache)
      stats = { changed: 0, errors: 0, failed: 0, duration: 0 }

      results.each do |result|
        if result[:error]
          handle_error_result(result, stats)
        elsif result[:changed]
          handle_changed_result(result, stats, cache)
        else
          handle_unchanged_result(result, cache)
        end
      end

      stats
    end

    def handle_error_result(result, stats)
      say "Error in #{result[:file]}: #{result[:error]}", :red
      stats[:errors] += 1
    end

    def handle_changed_result(result, stats, cache)
      stats[:changed] += 1

      if options[:check]
        say "#{result[:file]} needs formatting", :yellow
        stats[:failed] += 1
        show_diff(result[:file], result[:original], result[:formatted]) if options[:diff]
      elsif options[:diff]
        show_diff(result[:file], result[:original], result[:formatted])
      elsif options[:write]
        write_formatted_file(result, cache)
      else
        puts result[:formatted]
      end
    end

    def handle_unchanged_result(result, cache)
      say "✓ #{result[:file]} already formatted", :white if options[:verbose] && !options[:check]
      cache&.mark_formatted(result[:file])
    end

    def write_formatted_file(result, cache)
      File.write(result[:file], result[:formatted])
      say "✓ Formatted #{result[:file]}", :green unless options[:quiet]
      cache&.mark_formatted(result[:file])
    end

    def display_summary(stats, total_files)
      @last_stats = stats # Store for verbose details
      unchanged_count = total_files - stats[:changed] - stats[:errors]

      if stats[:errors].positive?
        display_error_summary(stats[:errors])
      elsif options[:check] && stats[:failed].positive?
        display_check_failed_summary(stats[:failed])
      elsif options[:quiet]
        display_quiet_summary(stats[:changed])
      else
        display_normal_summary(stats[:changed], unchanged_count, total_files)
      end

      display_verbose_details(total_files) if options[:verbose] && !options[:quiet]
    end

    def display_error_summary(error_count)
      say "\n✗ Failed: #{error_count} error(s) occurred", :red
    end

    def display_check_failed_summary(failed_count)
      say "\n✗ Check failed: #{failed_count} file(s) need formatting", :yellow
    end

    def display_quiet_summary(changed_count)
      say "✓ #{changed_count} files formatted", :cyan if changed_count.positive?
    end

    def display_normal_summary(changed_count, unchanged_count, total_files)
      if total_files == 1
        if changed_count.positive?
          say "\n✓ Formatted 1 file", :cyan
        else
          say "\n✓ File is already formatted", :cyan
        end
      else
        say "\n✓ Processed #{total_files} files", :cyan
        display_file_breakdown(changed_count, unchanged_count)
      end
    end

    def display_file_breakdown(changed_count, unchanged_count)
      return unless changed_count.positive? || unchanged_count.positive?

      parts = []
      parts << "#{changed_count} formatted" if changed_count.positive?
      parts << "#{unchanged_count} unchanged" if unchanged_count.positive?
      say "  (#{parts.join(', ')})", :white
    end

    def display_verbose_details(total_files)
      say "\nDetails:", :blue
      say "  Total files: #{total_files}", :blue

      # Duration is collected if available
      return unless defined?(@last_stats) && @last_stats[:total_duration]

      duration = @last_stats[:total_duration].round(2)
      say "  Total time: #{duration}s", :blue
      say "  Files/sec: #{(total_files / duration).round(1)}", :blue if duration.positive?
    end

    def should_exit_with_error?(stats)
      (options[:check] && stats[:failed].positive?) || stats[:errors].positive?
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
