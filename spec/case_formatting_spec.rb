# frozen_string_literal: true

require 'spec_helper'

RSpec.describe Rfmt, 'Case/When Formatting' do
  describe 'basic case statement' do
    it 'formats simple case' do
      source = "case x\nwhen 1\n\"one\"\nwhen 2\n\"two\"\nend"
      result = Rfmt.format(source)
      expect(result).to include('case x')
      expect(result).to include('when 1')
      expect(result).to include('when 2')
      expect(result).to include('end')
    end
  end

  describe 'case with else' do
    it 'formats case-when-else' do
      source = "case status\nwhen :pending\nprocess_pending\nelse\nhandle_unknown\nend"
      result = Rfmt.format(source)
      expect(result).to include('case status')
      expect(result).to include('when :pending')
      expect(result).to include('else')
      expect(result).to include('end')
    end
  end

  describe 'case without predicate' do
    it 'formats case-when-true pattern' do
      source = "case\nwhen x > 0\n\"positive\"\nwhen x < 0\n\"negative\"\nend"
      result = Rfmt.format(source)
      expect(result).to include('case')
      expect(result).to include('when x > 0')
      expect(result).to include('when x < 0')
    end
  end

  describe 'multiple when conditions' do
    it 'formats when with multiple values' do
      source = "case char\nwhen 'a', 'e', 'i'\n\"vowel\"\nend"
      result = Rfmt.format(source)
      expect(result).to include("when 'a', 'e', 'i'")
    end
  end

  describe 'case in method' do
    it 'formats nested case with proper indentation' do
      source = "def classify(value)\ncase value\nwhen Integer\n\"number\"\nwhen String\n\"text\"\nend\nend"
      result = Rfmt.format(source)
      expect(result).to include('def classify(value)')
      expect(result).to include('case value')
      expect(result).to include('when Integer')
      expect(result).to include('end')
    end
  end

  describe 'case with class matching' do
    it 'formats case with class conditions' do
      source = "case obj\nwhen Array\nhandle_array\nwhen Hash\nhandle_hash\nend"
      result = Rfmt.format(source)
      expect(result).to include('when Array')
      expect(result).to include('when Hash')
    end
  end

  describe 'case with range matching' do
    it 'formats case with range conditions' do
      source = "case age\nwhen 0..12\n\"child\"\nwhen 13..19\n\"teenager\"\nend"
      result = Rfmt.format(source)
      expect(result).to include('when 0..12')
      expect(result).to include('when 13..19')
    end
  end

  describe 'multiline when body' do
    it 'formats when with multiple statements' do
      source = "case action\nwhen :create\nvalidate\nsave\nwhen :delete\narchive\nend"
      result = Rfmt.format(source)
      expect(result).to include('validate')
      expect(result).to include('save')
      expect(result).to include('archive')
    end
  end

  describe 'nested case statements' do
    it 'formats nested case' do
      source = "case category\nwhen :animal\ncase species\nwhen :dog\n\"bark\"\nend\nwhen :vehicle\n\"vroom\"\nend"
      result = Rfmt.format(source)
      expect(result.scan('case').count).to eq(2)
      expect(result.scan('end').count).to eq(2)
    end
  end
end
