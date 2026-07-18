# frozen_string_literal: true

require 'spec_helper'
require_relative '../scripts/differential_check'

RSpec.describe DifferentialCheck::DiffReporter do
  describe '.unified_diff' do
    it 'returns nil for byte-identical outputs' do
      output = "def foo\n  1\nend\n"

      expect(described_class.unified_diff(output, output.dup)).to be_nil
    end

    it 'reports a deliberately perturbed pair as a unified hunk' do
      legacy = "def foo\n  1\nend\n"
      native = "def foo\n  2\nend\n"

      diff = described_class.unified_diff(legacy, native)

      expect(diff).to eq("@@ -2,1 +2,1 @@\n-  1\n+  2")
    end

    it 'reports a trailing addition' do
      legacy = "x = 1\n"
      native = "x = 1\ny = 2\n"

      diff = described_class.unified_diff(legacy, native)

      expect(diff).to eq("@@ -2,0 +2,1 @@\n+y = 2")
    end

    it 'does not leak common lines when one side purely inserts at the top' do
      legacy = "b\n"
      native = "a\nb\n"

      diff = described_class.unified_diff(legacy, native)

      expect(diff).to eq("@@ -1,0 +1,1 @@\n+a")
    end

    it 'makes a trailing-newline-only difference visible' do
      legacy = "a\nb"
      native = "a\nb\n"

      diff = described_class.unified_diff(legacy, native)

      expect(diff).to eq("@@ -2,1 +2,1 @@\n-b\n\\ No newline at end of file\n+b")
    end

    it 'truncates long diffs to the line limit' do
      legacy = Array.new(100) { |i| "a#{i}" }.join("\n")
      native = Array.new(100) { |i| "b#{i}" }.join("\n")

      diff = described_class.unified_diff(legacy, native, limit: 10)

      expect(diff.lines.size).to eq(11)
      expect(diff).to end_with('more diff lines truncated)')
    end
  end
end

RSpec.describe Rfmt, '.format_legacy' do
  it 'matches the native .format byte-for-byte on a simple program' do
    source = "def foo( a,b )\n  a+b\nend\n"

    expect(Rfmt.format(source)).to eq(Rfmt.format_legacy(source))
  end
end
