# frozen_string_literal: true

require 'spec_helper'

RSpec.describe Rfmt, 'Conditional Formatting' do
  describe 'if/elsif/else formatting' do
    it 'formats simple if statement' do
      source = <<~RUBY
        if x > 0
        puts "positive"
        end
      RUBY

      expected = <<~RUBY
        if x > 0
          puts "positive"
        end
      RUBY

      result = Rfmt.format(source)
      expect(result).to eq(expected)
    end

    it 'formats if with else' do
      source = <<~RUBY
        if x > 0
        puts "positive"
        else
        puts "not positive"
        end
      RUBY

      expected = <<~RUBY
        if x > 0
          puts "positive"
        else
          puts "not positive"
        end
      RUBY

      result = Rfmt.format(source)
      expect(result).to eq(expected)
    end

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

    it 'formats multiple elsif clauses' do
      source = <<~RUBY
        if x == 1
        puts "one"
        elsif x == 2
        puts "two"
        elsif x == 3
        puts "three"
        else
        puts "other"
        end
      RUBY

      expected = <<~RUBY
        if x == 1
          puts "one"
        elsif x == 2
          puts "two"
        elsif x == 3
          puts "three"
        else
          puts "other"
        end
      RUBY

      result = Rfmt.format(source)
      expect(result).to eq(expected)
    end

    it 'formats empty if body' do
      source = <<~RUBY
        if condition
        end
      RUBY

      expected = <<~RUBY
        if condition
        end
      RUBY

      result = Rfmt.format(source)
      expect(result).to eq(expected)
    end
  end

  describe 'unless formatting' do
    it 'formats simple unless statement' do
      source = <<~RUBY
        unless x.nil?
        puts "present"
        end
      RUBY

      expected = <<~RUBY
        unless x.nil?
          puts "present"
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
  end

  describe 'postfix if/unless formatting' do
    it 'preserves postfix if' do
      source = 'puts "yes" if x > 0'
      expected = "puts \"yes\" if x > 0\n"

      result = Rfmt.format(source)
      expect(result).to eq(expected)
    end

    it 'preserves postfix unless' do
      source = 'puts "yes" unless x.nil?'
      expected = "puts \"yes\" unless x.nil?\n"

      result = Rfmt.format(source)
      expect(result).to eq(expected)
    end

    it 'formats postfix if with complex statement' do
      source = 'do_something(a, b, c) if condition?'
      expected = "do_something(a, b, c) if condition?\n"

      result = Rfmt.format(source)
      expect(result).to eq(expected)
    end

    it 'formats postfix if in method' do
      source = <<~RUBY
        def foo
        return nil if x.nil?
        x * 2
        end
      RUBY

      expected = <<~RUBY
        def foo
          return nil if x.nil?
          x * 2
        end
      RUBY

      result = Rfmt.format(source)
      expect(result).to eq(expected)
    end

    it 'formats postfix unless in method' do
      source = <<~RUBY
        def validate
        return unless record
        process(record)
        end
      RUBY

      expected = <<~RUBY
        def validate
          return unless record
          process(record)
        end
      RUBY

      result = Rfmt.format(source)
      expect(result).to eq(expected)
    end
  end

  describe 'nested conditionals' do
    it 'formats nested if statements' do
      source = <<~RUBY
        if a > 0
        if b > 0
        puts "both positive"
        else
        puts "a positive, b not"
        end
        end
      RUBY

      expected = <<~RUBY
        if a > 0
          if b > 0
            puts "both positive"
          else
            puts "a positive, b not"
          end
        end
      RUBY

      result = Rfmt.format(source)
      expect(result).to eq(expected)
    end

    it 'formats deeply nested conditions' do
      source = <<~RUBY
        if a
        if b
        if c
        puts "all true"
        end
        end
        end
      RUBY

      expected = <<~RUBY
        if a
          if b
            if c
              puts "all true"
            end
          end
        end
      RUBY

      result = Rfmt.format(source)
      expect(result).to eq(expected)
    end

    it 'formats nested if/unless mix' do
      source = <<~RUBY
        unless a.nil?
        if b > 0
        puts "a present, b positive"
        else
        puts "a present, b not positive"
        end
        end
      RUBY

      expected = <<~RUBY
        unless a.nil?
          if b > 0
            puts "a present, b positive"
          else
            puts "a present, b not positive"
          end
        end
      RUBY

      result = Rfmt.format(source)
      expect(result).to eq(expected)
    end

    it 'formats if with nested elsif' do
      source = <<~RUBY
        if x > 0
        if y > 0
        puts "x and y positive"
        end
        elsif x < 0
        puts "x negative"
        end
      RUBY

      expected = <<~RUBY
        if x > 0
          if y > 0
            puts "x and y positive"
          end
        elsif x < 0
          puts "x negative"
        end
      RUBY

      result = Rfmt.format(source)
      expect(result).to eq(expected)
    end
  end

  describe 'conditionals in class/module context' do
    it 'formats if in method definition' do
      source = <<~RUBY
        class Validator
        def check(value)
        if value > 0
        :positive
        elsif value < 0
        :negative
        else
        :zero
        end
        end
        end
      RUBY

      expected = <<~RUBY
        class Validator
          def check(value)
            if value > 0
              :positive
            elsif value < 0
              :negative
            else
              :zero
            end
          end
        end
      RUBY

      result = Rfmt.format(source)
      expect(result).to eq(expected)
    end

    it 'formats multiple conditionals in method' do
      source = <<~RUBY
        def validate(record)
        return false unless record
        return false if record.invalid?
        if record.ready?
        process(record)
        else
        queue(record)
        end
        true
        end
      RUBY

      expected = <<~RUBY
        def validate(record)
          return false unless record
          return false if record.invalid?
          if record.ready?
            process(record)
          else
            queue(record)
          end
          true
        end
      RUBY

      result = Rfmt.format(source)
      expect(result).to eq(expected)
    end
  end

  describe 'edge cases' do
    it 'formats if with multiline condition' do
      source = <<~RUBY
        if x > 0 &&
           y > 0
        puts "both positive"
        end
      RUBY

      result = Rfmt.format(source)
      expect(result).to include('if x > 0 &&')
      expect(result).to include('puts "both positive"')
      expect(result).to include('end')
    end

    it 'formats if with method call condition' do
      source = <<~RUBY
        if user.authenticated? && user.active?
        grant_access
        end
      RUBY

      result = Rfmt.format(source)
      expect(result).to include('if user.authenticated? && user.active?')
      expect(result).to include('grant_access')
    end

    it 'formats postfix with return statement' do
      source = 'return :error if failed?'
      expected = "return :error if failed?\n"

      result = Rfmt.format(source)
      expect(result).to eq(expected)
    end

    it 'formats postfix with break statement in loop' do
      source = <<~RUBY
        loop do
          break if done
        end
      RUBY

      result = Rfmt.format(source)
      expect(result).to include('break if done')
    end

    it 'formats postfix with next statement in loop' do
      source = <<~RUBY
        items.each do |item|
          next unless valid
          process(item)
        end
      RUBY

      result = Rfmt.format(source)
      expect(result).to include('next unless valid')
    end
  end
end
