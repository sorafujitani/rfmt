# frozen_string_literal: true

require 'thor'
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

    desc 'format [FILES]', 'Format Ruby files'
    option :write, type: :boolean, default: true, desc: 'Write formatted output'
    option :check, type: :boolean, desc: "Check if files are formatted (don't write)"
    option :diff, type: :boolean, desc: 'Show diff of changes'
    option :diff_format, type: :string, default: 'unified', desc: 'Diff format: unified, side_by_side, or color'
    option :parallel, type: :boolean, default: true, desc: 'Process files in parallel'
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
        say "Skipped #{skipped} unchanged file(s) (cache hit)", :blue if skipped > 0 && options[:verbose]
      end

      if files.empty?
        say 'All files are already formatted', :green
        return
      end

      say "Formatting #{files.size} file(s)...", :blue if options[:verbose]

      results = if options[:parallel] && files.size > 1
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
    def init
      config_file = '.rfmt.yml'

      if File.exist?(config_file) && !options[:force]
        say 'Configuration file already exists. Use --force to overwrite.', :yellow
        return
      end

      template_config = {
        'version' => '1.0',
        'formatting' => {
          'line_length' => 100,
          'indent_width' => 2,
          'indent_style' => 'spaces'
        },
        'include' => ['**/*.rb'],
        'exclude' => ['vendor/**/*', 'tmp/**/*']
      }

      File.write(config_file, YAML.dump(template_config))
      say "Created #{config_file}", :green
    end

    private

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
            say "Formatted #{result[:file]}", :green if options[:verbose]

            # Update cache after successful write
            cache&.mark_formatted(result[:file])
          else
            puts result[:formatted]
          end
        else
          say "#{result[:file]} already formatted", :blue if options[:verbose]

          # Update cache even if no changes (file was checked)
          cache&.mark_formatted(result[:file])
        end
      end

      # Save cache to disk
      cache&.save

      # Summary
      say "\n#{results.size} file(s) processed", :blue if options[:verbose]
      say "#{changed_count} file(s) changed", :yellow if changed_count.positive? && options[:verbose]
      say "#{error_count} error(s)", :red if error_count.positive?

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
