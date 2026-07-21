# frozen_string_literal: true

require 'prism'
require_relative '../lib/rfmt'

module CorpusCheck
  CORPUS_GLOBS = ['lib/**/*.rb', 'spec/**/*.rb', 'scripts/**/*.rb'].freeze

  # Files with known formatter bugs, skipped with a warning (exit stays 0).
  KNOWN_FAILURES = [].freeze

  # Structural AST equality that ignores source positions, so that a
  # reformatted file can be compared against its original.
  module AstComparator
    module_function

    # Some semantics live only in flags plus an operator token, not in
    # deconstruct_keys values: `a.b` vs `a&.b`, `1..5` vs `1...5`. Prism
    # exposes no public flags reader, so compare these tokens by slice.
    OPERATOR_LOC_KEYS = %i[operator_loc call_operator_loc].freeze
    # Regex options (/x/i vs /x/) live in the closing token.
    REGEXP_NODES = [Prism::RegularExpressionNode, Prism::InterpolatedRegularExpressionNode].freeze

    def equivalent?(left, right)
      equivalent_values?(left, right)
    end

    def equivalent_nodes?(left, right)
      return false unless left.instance_of?(right.class)

      left_keys = left.deconstruct_keys(nil)
      right_keys = right.deconstruct_keys(nil)

      left_keys.all? do |key, left_value|
        right_value = right_keys[key]
        next true if key == :node_id
        next slice_pair_equal?(left_value, right_value) if OPERATOR_LOC_KEYS.include?(key)
        next slice_pair_equal?(left_value, right_value) if key == :closing_loc && REGEXP_NODES.include?(left.class)
        next true if location_pair?(left_value, right_value)

        equivalent_values?(left_value, right_value)
      end
    end

    def slice_pair_equal?(left, right)
      return left == right unless left.is_a?(Prism::Location) && right.is_a?(Prism::Location)

      left.slice == right.slice
    end

    def equivalent_values?(left, right)
      case left
      when Prism::Node
        right.is_a?(Prism::Node) && equivalent_nodes?(left, right)
      when Array
        right.is_a?(Array) &&
          left.size == right.size &&
          left.zip(right).all? { |l, r| equivalent_values?(l, r) }
      else
        left == right
      end
    end

    def location_pair?(left, right)
      [left, right].any? { |value| value.is_a?(Prism::Location) } &&
        [left, right].all? { |value| value.is_a?(Prism::Location) || value.nil? }
    end
  end

  class Runner
    def initialize(globs: CORPUS_GLOBS, root: File.expand_path('..', __dir__))
      @globs = globs
      @root = root
      @checked = 0
      @passed = 0
      @skipped = 0
      @failed = 0
    end

    def run
      Dir.chdir(@root) do
        files = @globs.flat_map { |glob| Dir[glob] }.uniq.sort
        stale = KNOWN_FAILURES - files
        unless stale.empty?
          puts "FAIL: KNOWN_FAILURES entries not in corpus: #{stale.join(', ')}"
          @failed += stale.size
        end
        files.each { |path| check_file(path) }
      end

      puts "\nchecked: #{@checked}, passed: #{@passed}, skipped: #{@skipped}, failed: #{@failed}"
      @failed.zero? ? 0 : 1
    end

    private

    def check_file(path)
      source = File.read(path)
      source_result = Prism.parse(source)
      unless source_result.success?
        warn "SKIP (source does not parse): #{path}"
        @skipped += 1
        return
      end

      @checked += 1
      failures = property_failures(path, source, source_result)
      record(path, failures)
    rescue StandardError => e
      @checked += 1
      record(path, ["error during check: #{e.class}: #{e.message}"])
    end

    # Known failures still run every property so a fixed file surfaces as
    # XPASS instead of rotting in the list forever.
    def record(path, failures)
      known = KNOWN_FAILURES.include?(path)
      if failures.empty? && known
        @failed += 1
        puts "XPASS: #{path}: all properties pass; remove it from KNOWN_FAILURES"
      elsif failures.empty?
        @passed += 1
      elsif known
        @skipped += 1
        warn "SKIP (known failure): #{path}"
      else
        @failed += 1
        failures.each { |message| puts "FAIL: #{path}: #{message}" }
      end
    end

    def property_failures(_path, source, source_result)
      # The guard in Rfmt.format raises before invalid output can reach us
      begin
        formatted = Rfmt.format(source)
      rescue Rfmt::ValidationError => e
        return ["syntactic validity: #{e.message}"]
      end

      formatted_result = Prism.parse(formatted)

      failures = []
      failures << idempotency_failure(formatted)
      unless AstComparator.equivalent?(source_result.value, formatted_result.value)
        failures << 'AST equivalence: formatted output changed program structure'
      end
      # Comments are not part of the AST, so a dropped comment passes every other property
      if source_result.comments.size != formatted_result.comments.size
        failures << "comment preservation: #{source_result.comments.size} comments in source, " \
                    "#{formatted_result.comments.size} in output"
      end
      # The __END__ data section is outside the AST, so no other property sees its loss
      if source_result.data_loc&.slice != formatted_result.data_loc&.slice
        failures << 'data section preservation: __END__ content changed or dropped'
      end
      failures.compact
    end

    def idempotency_failure(formatted)
      reformatted = Rfmt.format(formatted)
      return nil if reformatted == formatted

      "idempotency: second format pass changed output\n#{first_diff_excerpt(formatted, reformatted)}"
    end

    def first_diff_excerpt(formatted, reformatted)
      formatted_lines = formatted.lines
      reformatted_lines = reformatted.lines
      index = formatted_lines.zip(reformatted_lines).index { |a, b| a != b } || formatted_lines.size
      [
        "    line #{index + 1}:",
        "    - #{formatted_lines[index].to_s.chomp}",
        "    + #{reformatted_lines[index].to_s.chomp}"
      ].join("\n")
    end
  end
end

exit CorpusCheck::Runner.new.run if __FILE__ == $PROGRAM_NAME
