# frozen_string_literal: true

require 'benchmark'
require_relative '../lib/kenshin'

# In-process throughput benchmark for Kenshin.format. CLI wall-clock is dominated
# by Ruby VM startup, so pipeline changes must be measured in-process.
module BenchFormat
  ROUNDS = Integer(ENV.fetch('BENCH_ROUNDS', '5'), 10)
  GLOBS = ['lib/**/*.rb'].freeze

  module_function

  def run
    root = File.expand_path('..', __dir__)
    files = Dir.chdir(root) { GLOBS.flat_map { |glob| Dir[glob] }.sort }
    sources = files.map { |f| File.read(File.join(root, f)) }

    sources.each { |s| safe_format(s) }

    total = Benchmark.realtime do
      ROUNDS.times { sources.each { |s| safe_format(s) } }
    end

    runs = sources.size * ROUNDS
    puts format(
      'files=%<files>d rounds=%<rounds>d total=%<total>.1fms avg=%<avg>.3fms/file',
      files: sources.size, rounds: ROUNDS, total: total * 1000, avg: total * 1000 / runs
    )
  end

  def safe_format(source)
    Kenshin.format(source)
  rescue Kenshin::Error
    nil
  end
end

BenchFormat.run if __FILE__ == $PROGRAM_NAME
