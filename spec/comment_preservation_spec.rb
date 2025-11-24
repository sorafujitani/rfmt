# frozen_string_literal: true

require 'spec_helper'

RSpec.describe 'Comment Preservation' do
  describe 'top-level comments' do
    it 'preserves single line comment before class' do
      source = <<~RUBY
        # This is a comment
        class Foo
        end
      RUBY

      result = Rfmt.format(source)
      expect(result).to include('# This is a comment')
      expect(result).to include('class Foo')
    end

    it 'preserves multiple line comments' do
      source = <<~RUBY
        # First comment
        # Second comment
        class Foo
        end
      RUBY

      result = Rfmt.format(source)
      expect(result).to include('# First comment')
      expect(result).to include('# Second comment')
    end
  end

  describe 'method comments' do
    it 'preserves comment before method' do
      source = <<~RUBY
        class Foo
          # Method documentation
          def bar
          end
        end
      RUBY

      result = Rfmt.format(source)
      expect(result).to include('# Method documentation')
    end

    it 'preserves inline comment in method body' do
      source = <<~RUBY
        class Foo
          def bar
            x = 1 # inline comment
          end
        end
      RUBY

      result = Rfmt.format(source)
      expect(result).to include('x = 1 # inline comment')
    end
  end

  describe 'nested comments' do
    it 'preserves comments at different indentation levels' do
      source = <<~RUBY
        class Foo
          # Method 1
          def first
            42
          end

          # Method 2
          def second
            # inner comment
            43
          end
        end
      RUBY

      result = Rfmt.format(source)
      expect(result).to include('# Method 1')
      expect(result).to include('# Method 2')
      expect(result).to include('# inner comment')
    end
  end

  describe 'complex scenarios' do
    it 'preserves all comments in a complex class' do
      source = <<~RUBY
        # Class header
        class MyClass
          # Constant
          CONST = 42

          # Initialize
          def initialize(name)
            @name = name # instance variable
          end

          # Greet method
          def greet
            puts "Hello" # print greeting
          end
        end
      RUBY

      result = Rfmt.format(source)
      expect(result).to include('# Class header')
      expect(result).to include('# Constant')
      expect(result).to include('# Initialize')
      expect(result).to include('@name = name # instance variable')
      expect(result).to include('# Greet method')
      expect(result).to include('puts "Hello" # print greeting')
    end
  end
end
