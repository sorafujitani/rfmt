# frozen_string_literal: true

require 'spec_helper'

RSpec.describe Rfmt, 'Conditional Formatting' do
  it 'formats if with elsif and else' do
    source = <<~RUBY
      if x > 0
      puts "positive"
      elsif x < 0
      puts "negative"
      else
      puts "zero"
      end
    RUBY

    expected = <<~RUBY
      if x > 0
        puts "positive"
      elsif x < 0
        puts "negative"
      else
        puts "zero"
      end
    RUBY

    result = Rfmt.format(source)
    expect(result).to eq(expected)
  end

  it 'formats unless with else' do
    source = <<~RUBY
      unless x > 0
      puts "not positive"
      else
      puts "positive"
      end
    RUBY

    expected = <<~RUBY
      unless x > 0
        puts "not positive"
      else
        puts "positive"
      end
    RUBY

    result = Rfmt.format(source)
    expect(result).to eq(expected)
  end

  it 'preserves postfix if/unless' do
    source = 'puts "yes" if x > 0'
    result = Rfmt.format(source)
    expect(result.strip).to eq('puts "yes" if x > 0')
  end

  it 'formats nested conditionals with proper indentation' do
    source = <<~RUBY
      if a > 0
      if b > 0
      puts "both positive"
      end
      end
    RUBY

    expected = <<~RUBY
      if a > 0
        if b > 0
          puts "both positive"
        end
      end
    RUBY

    result = Rfmt.format(source)
    expect(result).to eq(expected)
  end

  describe 'postfix if/unless with inline comments (Issue #87)' do
    it 'preserves inline comment after postfix if' do
      source = "some_method if condition # comment\n"
      result = Rfmt.format(source)
      expect(result).to eq("some_method if condition # comment\n")
    end

    it 'preserves inline comment after postfix unless' do
      source = "some_method unless condition # comment\n"
      result = Rfmt.format(source)
      expect(result).to eq("some_method unless condition # comment\n")
    end

    it 'preserves tool directive comment (steep:ignore) after postfix if' do
      source = "some_method if condition # steep:ignore\n"
      result = Rfmt.format(source)
      expect(result).to eq("some_method if condition # steep:ignore\n")
    end

    it 'preserves postfix if without comment (regression)' do
      source = "some_method if condition\n"
      result = Rfmt.format(source)
      expect(result).to eq("some_method if condition\n")
    end

    it 'preserves postfix unless without comment (regression)' do
      source = "some_method unless condition\n"
      result = Rfmt.format(source)
      expect(result).to eq("some_method unless condition\n")
    end
  end

  it 'formats conditionals in class/method context' do
    source = <<~RUBY
      class Validator
      def check(value)
      if value > 0
      :positive
      else
      :not_positive
      end
      end
      end
    RUBY

    expected = <<~RUBY
      class Validator
        def check(value)
          if value > 0
            :positive
          else
            :not_positive
          end
        end
      end
    RUBY

    result = Rfmt.format(source)
    expect(result).to eq(expected)
  end
end
