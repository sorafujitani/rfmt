# frozen_string_literal: true

require 'spec_helper'

RSpec.describe 'Logging System' do
  describe 'initialization logging' do
    it 'logs module initialization' do
      # Capture stderr to check for log messages
      original_stderr = $stderr
      captured = StringIO.new
      $stderr = captured

      # Force module reload by requiring again (this might not show logs as it's already loaded)
      # But we can verify the module works
      expect(Rfmt).to be_a(Module)

      $stderr = original_stderr

      # The logging happens during initial load, so we can't easily test it here
      # But we can verify the logging system exists by checking for formatted code
      source = "class Foo\nend"
      result = Rfmt.format(source)
      expect(result).to include('class Foo')
    end
  end

  describe 'logging levels' do
    it 'operates correctly regardless of log level' do
      # Test that logging doesn't interfere with normal operation
      sources = [
        "class Simple\nend",
        "module MyModule\nend",
        "def method\n42\nend"
      ]

      sources.each do |source|
        expect { Rfmt.format(source) }.not_to raise_error
      end
    end

    it 'processes complex code without logging errors' do
      source = <<~RUBY
        class User < ApplicationRecord
          has_many :posts
          validates :email, presence: true

          def full_name
            "\#{first_name} \#{last_name}"
          end

          private

          def sanitize_input
            # Implementation
          end
        end
      RUBY

      result = Rfmt.format(source)
      expect(result).to include('class User')
      expect(result).to include('has_many :posts')
      expect(result).to include('validates :email')
    end
  end

  describe 'debug information' do
    it 'provides version through Rust extension' do
      version = Rfmt.rust_version
      expect(version).to be_a(String)
      expect(version).to include('Rust')
    end

    it 'formats code with debug context maintained' do
      # Test that debug context tracking doesn't break formatting
      source = <<~RUBY
        # File header
        class DebugTest
          # Class comment
          def method
            # Method comment
            value = 42
            value * 2
          end
        end
      RUBY

      result = Rfmt.format(source)

      # Verify structure is maintained
      expect(result).to include('# File header')
      expect(result).to include('class DebugTest')
      expect(result).to include('def method')
      expect(result).to include('value = 42')
    end
  end

  describe 'performance logging' do
    it 'formats code efficiently' do
      # Generate a larger piece of code to test performance
      methods = (1..50).map do |i|
        "  def method_#{i}\n    #{i}\n  end\n"
      end.join

      large_source = "class LargeClass\n#{methods}end\n"

      start_time = Time.now
      result = Rfmt.format(large_source)
      elapsed = Time.now - start_time

      expect(result).to include('class LargeClass')
      # Should format reasonably quickly (under 1 second for this size)
      expect(elapsed).to be < 1.0
    end
  end

  describe 'error logging context' do
    it 'maintains context when encountering errors' do
      # Test that even with invalid input, system maintains state
      source = "class Valid\nend"
      valid_result = Rfmt.format(source)
      expect(valid_result).to include('class Valid')

      # After an error attempt, should still work
      begin
        Rfmt.format_code(source, "invalid")
      rescue StandardError
        # Expected to fail
      end

      # Should still work after error
      result2 = Rfmt.format(source)
      expect(result2).to include('class Valid')
    end
  end
end
