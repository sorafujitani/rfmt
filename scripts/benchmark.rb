#!/usr/bin/env ruby
# frozen_string_literal: true

require 'benchmark'
require 'fileutils'
require 'json'

class SimpleFormatterBenchmark
  BENCHMARK_RUNS = 10

  def initialize(rails_path)
    @rails_path = File.expand_path(rails_path)
    @results = {
      metadata: collect_metadata,
      single_file_benchmark: nil,
      directory_benchmark: nil,
      full_project_benchmark: nil
    }

    validate_environment!
  end

  def run_all
    puts '=' * 80
    puts 'rfmt vs RuboCop Performance Benchmark'
    puts '=' * 80
    puts "Project: #{@rails_path}"
    puts "Ruby: #{RUBY_VERSION}"
    puts "rfmt: #{rfmt_version}"
    puts "RuboCop: #{rubocop_version}"
    puts "Runs: #{BENCHMARK_RUNS}"
    puts '=' * 80
    puts

    benchmark_single_file
    benchmark_directory
    benchmark_full_project

    save_results
    print_summary
  end

  private

  def validate_environment!
    abort "Error: Project not found at #{@rails_path}" unless Dir.exist?(@rails_path)

    abort 'Error: rfmt not found' unless system('which rfmt > /dev/null 2>&1')

    abort 'Error: rubocop not found' unless system('which rubocop > /dev/null 2>&1')

    ruby_files = Dir.glob("#{@rails_path}/**/*.rb")
    abort 'Error: No Ruby files found' if ruby_files.empty?

    puts "Found #{ruby_files.size} Ruby files\n\n"
  end

  def collect_metadata
    {
      date: Time.now.iso8601,
      runs: BENCHMARK_RUNS,
      system: {
        os: `uname -s`.strip,
        os_version: `uname -r`.strip,
        cpu: `uname -m`.strip,
        ruby_version: RUBY_VERSION,
        ruby_platform: RUBY_PLATFORM
      },
      tools: {
        rfmt_version: rfmt_version,
        rubocop_version: rubocop_version
      },
      project: {
        path: @rails_path,
        total_files: Dir.glob("#{@rails_path}/**/*.rb").size,
        total_lines: count_total_lines
      }
    }
  end

  def rfmt_version
    @rfmt_version ||= `rfmt --version 2>&1`.strip.split("\n").first || 'unknown'
  rescue StandardError
    'unknown'
  end

  def rubocop_version
    @rubocop_version ||= `rubocop --version`.strip.split("\n").first || 'unknown'
  rescue StandardError
    'unknown'
  end

  def count_total_lines
    files = Dir.glob("#{@rails_path}/**/*.rb")
    files.sum do |f|
      File.readlines(f).size
    rescue StandardError
      0
    end
  end

  def benchmark_single_file
    puts 'Benchmark 1: Single File Performance'
    puts '-' * 80

    # Select a medium-sized file for testing
    all_files = Dir.glob("#{@rails_path}/**/*.rb")
    sorted = all_files.sort_by { |f| File.size(f) }
    test_file = sorted[sorted.size / 2] # Median file

    size = File.size(test_file)
    lines = File.readlines(test_file).size

    puts "File: #{File.basename(test_file)}"
    puts "Size: #{format_bytes(size)}, Lines: #{lines}"
    puts

    backup_dir = "#{@rails_path}/.benchmark_backup"

    begin
      FileUtils.mkdir_p(backup_dir)
      backup_file = "#{backup_dir}/test_file.rb"
      FileUtils.cp(test_file, backup_file)

      # Benchmark rfmt
      puts 'Testing rfmt...'
      rfmt_times = []
      BENCHMARK_RUNS.times do |_i|
        FileUtils.cp(backup_file, test_file)

        time = Benchmark.realtime do
          system("rfmt format #{test_file} > /dev/null 2>&1")
        end
        rfmt_times << time
        print '.'
      end
      puts ' done'

      # Benchmark RuboCop
      puts 'Testing RuboCop...'
      rubocop_times = []
      BENCHMARK_RUNS.times do |_i|
        FileUtils.cp(backup_file, test_file)

        time = Benchmark.realtime do
          system("rubocop -A #{test_file} > /dev/null 2>&1")
        end
        rubocop_times << time
        print '.'
      end
      puts ' done'

      # Restore original
      FileUtils.cp(backup_file, test_file)

      @results[:single_file_benchmark] = {
        file: File.basename(test_file),
        size_bytes: size,
        lines: lines,
        rfmt: calculate_stats(rfmt_times),
        rubocop: calculate_stats(rubocop_times),
        ratio: average(rubocop_times) / average(rfmt_times)
      }

      print_benchmark_result('Single File', rfmt_times, rubocop_times)
    ensure
      FileUtils.rm_rf(backup_dir)
    end
  end

  def benchmark_directory
    puts "\nBenchmark 2: Directory Performance"
    puts '-' * 80

    # Find app/models or similar directory with multiple files
    test_dirs = %w[app/models app/controllers lib].map { |d| File.join(@rails_path, d) }
    test_dir = test_dirs.find { |d| Dir.exist?(d) && Dir.glob("#{d}/**/*.rb").size >= 5 }

    unless test_dir
      puts "Skipping: No suitable directory found\n"
      return
    end

    files = Dir.glob("#{test_dir}/**/*.rb")
    puts "Directory: #{File.basename(test_dir)}"
    puts "Files: #{files.size}"
    puts

    backup_dir = "#{@rails_path}/.benchmark_backup_dir"

    begin
      FileUtils.mkdir_p(backup_dir)

      # Backup all files
      files.each do |file|
        backup_file = File.join(backup_dir, File.basename(file))
        FileUtils.cp(file, backup_file)
      end

      # Benchmark rfmt
      puts 'Testing rfmt...'
      rfmt_times = []
      BENCHMARK_RUNS.times do
        # Restore all files
        files.each do |file|
          backup_file = File.join(backup_dir, File.basename(file))
          FileUtils.cp(backup_file, file)
        end

        time = Benchmark.realtime do
          system("rfmt format #{test_dir} > /dev/null 2>&1")
        end
        rfmt_times << time
        print '.'
      end
      puts ' done'

      # Benchmark RuboCop
      puts 'Testing RuboCop...'
      rubocop_times = []
      BENCHMARK_RUNS.times do
        # Restore all files
        files.each do |file|
          backup_file = File.join(backup_dir, File.basename(file))
          FileUtils.cp(backup_file, file)
        end

        time = Benchmark.realtime do
          system("rubocop -A #{test_dir} > /dev/null 2>&1")
        end
        rubocop_times << time
        print '.'
      end
      puts ' done'

      # Restore all files
      files.each do |file|
        backup_file = File.join(backup_dir, File.basename(file))
        FileUtils.cp(backup_file, file)
      end

      @results[:directory_benchmark] = {
        directory: File.basename(test_dir),
        file_count: files.size,
        rfmt: calculate_stats(rfmt_times),
        rubocop: calculate_stats(rubocop_times),
        ratio: average(rubocop_times) / average(rfmt_times)
      }

      print_benchmark_result('Directory', rfmt_times, rubocop_times)
    ensure
      FileUtils.rm_rf(backup_dir)
    end
  end

  def benchmark_full_project
    puts "\nBenchmark 3: Full Project (Check Mode)"
    puts '-' * 80

    all_files = Dir.glob("#{@rails_path}/**/*.rb")
    puts "Files: #{all_files.size}"
    puts

    # rfmt check
    puts 'Testing rfmt check...'
    rfmt_times = []
    BENCHMARK_RUNS.times do
      time = Benchmark.realtime do
        system("rfmt check #{@rails_path} > /dev/null 2>&1")
      end
      rfmt_times << time
      print '.'
    end
    puts ' done'

    # RuboCop check
    puts 'Testing RuboCop check...'
    rubocop_times = []
    BENCHMARK_RUNS.times do
      time = Benchmark.realtime do
        system("rubocop #{@rails_path} > /dev/null 2>&1")
      end
      rubocop_times << time
      print '.'
    end
    puts ' done'

    @results[:full_project_benchmark] = {
      total_files: all_files.size,
      rfmt: calculate_stats(rfmt_times),
      rubocop: calculate_stats(rubocop_times),
      ratio: average(rubocop_times) / average(rfmt_times)
    }

    print_benchmark_result('Full Project', rfmt_times, rubocop_times)
  end

  def calculate_stats(times)
    {
      times: times,
      avg: average(times),
      median: median(times),
      min: times.min,
      max: times.max,
      stddev: std_dev(times)
    }
  end

  def average(times)
    times.sum / times.size.to_f
  end

  def median(times)
    sorted = times.sort
    mid = sorted.size / 2
    if sorted.size.odd?
      sorted[mid]
    else
      (sorted[mid - 1] + sorted[mid]) / 2.0
    end
  end

  def std_dev(times)
    return 0 if times.size < 2

    avg = average(times)
    variance = times.sum { |t| (t - avg)**2 } / (times.size - 1)
    Math.sqrt(variance)
  end

  def format_time(seconds)
    if seconds < 1
      "#{(seconds * 1000).round(1)}ms"
    else
      "#{seconds.round(3)}s"
    end
  end

  def format_bytes(bytes)
    if bytes < 1024
      "#{bytes}B"
    elsif bytes < 1024 * 1024
      "#{(bytes / 1024.0).round(1)}KB"
    else
      "#{(bytes / 1024.0 / 1024.0).round(1)}MB"
    end
  end

  def print_benchmark_result(_label, rfmt_times, rubocop_times)
    rfmt_avg = average(rfmt_times)
    rubocop_avg = average(rubocop_times)
    ratio = rubocop_avg / rfmt_avg

    puts
    puts 'Results:'
    puts "  rfmt:    avg=#{format_time(rfmt_avg)}, median=#{format_time(median(rfmt_times))}, σ=#{format_time(std_dev(rfmt_times))}"
    puts "  RuboCop: avg=#{format_time(rubocop_avg)}, median=#{format_time(median(rubocop_times))}, σ=#{format_time(std_dev(rubocop_times))}"
    puts "  Ratio: #{ratio.round(2)}x"
    puts
  end

  def save_results
    output_dir = File.expand_path('docs/benchmark', __dir__)
    FileUtils.mkdir_p(output_dir)

    timestamp = Time.now.strftime('%Y%m%d_%H%M%S')
    result_file = File.join(output_dir, "results_#{timestamp}.json")

    File.write(result_file, JSON.pretty_generate(@results))

    latest_file = File.join(output_dir, 'results.json')
    File.write(latest_file, JSON.pretty_generate(@results))

    puts 'Results saved:'
    puts "  #{result_file}"
    puts "  #{latest_file}"
    puts
  end

  def print_summary
    puts '=' * 80
    puts 'Summary'
    puts '=' * 80

    puts "Single File: #{@results[:single_file_benchmark][:ratio].round(2)}x" if @results[:single_file_benchmark]

    puts "Directory: #{@results[:directory_benchmark][:ratio].round(2)}x" if @results[:directory_benchmark]

    puts "Full Project: #{@results[:full_project_benchmark][:ratio].round(2)}x" if @results[:full_project_benchmark]

    puts '=' * 80
  end
end

# Main execution
if ARGV[0]
  begin
    benchmark = SimpleFormatterBenchmark.new(ARGV[0])
    benchmark.run_all
  rescue Interrupt
    puts "\n\nBenchmark interrupted"
    exit 1
  rescue StandardError => e
    puts "\nError: #{e.message}"
    puts e.backtrace.join("\n") if ENV['DEBUG']
    exit 1
  end
else
  puts 'Usage: ruby run_benchmark_simple.rb /path/to/rails/project'
  exit 1
end
