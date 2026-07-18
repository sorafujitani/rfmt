# frozen_string_literal: true

require 'prism'
require_relative '../lib/rfmt'
require_relative 'corpus_check'

# Temporary for the prism migration: proves Rfmt.format_native emits
# byte-identical output to Rfmt.format across the corpus. Deleted in
# phase 7 together with format_native.
module DifferentialCheck
  CORPUS_GLOBS = (CorpusCheck::CORPUS_GLOBS + ['ext/rfmt/tests/fixtures/parity/*.rb']).freeze

  module DiffReporter
    MAX_DIFF_LINES = 40

    module_function

    def unified_diff(legacy, native, limit: MAX_DIFF_LINES)
      return nil if legacy == native

      legacy_lines = legacy.lines
      native_lines = native.lines
      first = common_prefix_size(legacy_lines, native_lines)
      last_legacy, last_native = trim_common_suffix(legacy_lines, native_lines, first)

      hunk = ["@@ -#{first + 1},#{last_legacy - first + 1} +#{first + 1},#{last_native - first + 1} @@"]
      hunk += changed_slice(legacy_lines, first, last_legacy).map { |line| render('-', line) }
      hunk += changed_slice(native_lines, first, last_native).map { |line| render('+', line) }
      return hunk.join("\n") if hunk.size <= limit

      "#{hunk.first(limit).join("\n")}\n... (#{hunk.size - limit} more diff lines truncated)"
    end

    # Ruby wraps a negative end index around, so an empty range must be
    # returned explicitly (one side purely inserting lines hits this).
    def changed_slice(lines, first, last)
      return [] if last < first

      lines[first..last]
    end

    def render(sign, line)
      return "#{sign}#{line.chomp}" if line.end_with?("\n")

      "#{sign}#{line}\n\\ No newline at end of file"
    end

    def common_prefix_size(left, right)
      size = 0
      size += 1 while size < left.size && size < right.size && left[size] == right[size]
      size
    end

    def trim_common_suffix(left, right, prefix_size)
      last_left = left.size - 1
      last_right = right.size - 1
      while last_left >= prefix_size && last_right >= prefix_size && left[last_left] == right[last_right]
        last_left -= 1
        last_right -= 1
      end
      [last_left, last_right]
    end
  end

  class Runner
    def initialize(globs: CORPUS_GLOBS, root: File.expand_path('..', __dir__))
      @globs = globs
      @root = root
      @checked = 0
      @matched = 0
      @skipped = 0
      @mismatched = 0
      @errored = 0
    end

    def run
      Dir.chdir(@root) do
        # A glob matching nothing means the safety net silently shrank
        # (e.g. the parity fixtures moved), so it fails instead of passing.
        @globs.each do |glob|
          next unless Dir[glob].empty?

          @errored += 1
          puts "ERROR: corpus glob matched no files: #{glob}"
        end
        files = @globs.flat_map { |glob| Dir[glob] }.uniq.sort
        files.each { |path| check_file(path) }
      end

      puts "\nchecked: #{@checked}, matched: #{@matched}, skipped: #{@skipped}, " \
           "mismatched: #{@mismatched}, errored: #{@errored}"
      @checked.positive? && @mismatched.zero? && @errored.zero? ? 0 : 1
    end

    private

    def check_file(path)
      source = File.read(path)
      unless Prism.parse(source).success?
        warn "SKIP (source does not parse): #{path}"
        @skipped += 1
        return
      end

      @checked += 1
      compare(path, source)
    end

    def compare(path, source)
      legacy = Rfmt.format(source)
      native = Rfmt.format_native(source)

      diff = DiffReporter.unified_diff(legacy, native)
      if diff.nil?
        @matched += 1
      else
        @mismatched += 1
        puts "MISMATCH: #{path}\n#{diff}"
      end
    rescue StandardError => e
      @errored += 1
      puts "ERROR: #{path}: #{e.class}: #{e.message}"
    end
  end
end

exit DifferentialCheck::Runner.new.run if __FILE__ == $PROGRAM_NAME
