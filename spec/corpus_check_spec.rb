# frozen_string_literal: true

require 'spec_helper'
require 'prism'
require_relative '../scripts/corpus_check'

RSpec.describe CorpusCheck::AstComparator do
  def parse(source)
    Prism.parse(source).value
  end

  describe '.equivalent?' do
    it 'accepts semantically identical sources with different quoting' do
      single = parse("x = 'hello'\n")
      double = parse(%(x = "hello"\n))

      expect(described_class.equivalent?(single, double)).to be true
    end

    it 'accepts identical sources with different layout' do
      compact = parse('def foo(a, b) = a + b')
      expanded = parse("def foo(a, b) = a + b\n")

      expect(described_class.equivalent?(compact, expanded)).to be true
    end

    it 'rejects a dropped statement' do
      full = parse("x = 1\ny = 2\n")
      dropped = parse("x = 1\n")

      expect(described_class.equivalent?(full, dropped)).to be false
    end

    it 'rejects a changed literal' do
      original = parse("x = 1\n")
      changed = parse("x = 2\n")

      expect(described_class.equivalent?(original, changed)).to be false
    end

    it 'rejects safe navigation downgraded to a plain call' do
      safe = parse("a&.b\n")
      plain = parse("a.b\n")

      expect(described_class.equivalent?(safe, plain)).to be false
    end

    it 'rejects an exclusive range changed to inclusive' do
      exclusive = parse("1...5\n")
      inclusive = parse("1..5\n")

      expect(described_class.equivalent?(exclusive, inclusive)).to be false
    end

    it 'rejects dropped regexp options' do
      insensitive = parse("/x/i\n")
      sensitive = parse("/x/\n")

      expect(described_class.equivalent?(insensitive, sensitive)).to be false
    end
  end
end
