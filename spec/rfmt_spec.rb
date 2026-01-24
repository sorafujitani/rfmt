# frozen_string_literal: true

require 'spec_helper'

RSpec.describe Rfmt do
  describe '.format' do
    it 'formats simple Ruby code' do
      source = <<~RUBY
        class Foo
        def bar
        42
        end
        end
      RUBY

      result = Rfmt.format(source)

      expect(result).to be_a(String)
      expect(result).to include('class Foo')
      expect(result).to include('def bar')
      expect(result).to include('end')
    end

    it 'raises error for invalid Ruby syntax' do
      invalid_source = 'class Foo def'

      expect do
        Rfmt.format(invalid_source)
      end.to raise_error(Rfmt::Error)
    end

    it 'formats Rails migration with versioned superclass' do
      source = <<~RUBY
        class AddProfileToUsers < ActiveRecord::Migration[8.1]
          def change
            add_column :users, :profile, :text
          end
        end
      RUBY

      result = Rfmt.format(source)

      expect(result).to include('class AddProfileToUsers < ActiveRecord::Migration[8.1]')
      expect(result).to include('def change')
      expect(result).not_to include('Prism::CallNode')
    end

    it 'formats method with required keyword parameters' do
      source = <<~RUBY
        def initialize(frame:, form:, tag_map:)
          @frame = frame
          @form = form
          @tag_map = tag_map
        end
      RUBY

      result = Rfmt.format(source)

      expect(result).to include('def initialize(frame:, form:, tag_map:)')
      expect(result).not_to match(/frame:\s+form:\s+tag_map:/)
      expect(result).to include('@frame = frame')
    end

    it 'formats method with optional keyword parameters' do
      source = <<~RUBY
        def configure(timeout: 30, retries: 3)
          @timeout = timeout
          @retries = retries
        end
      RUBY

      result = Rfmt.format(source)

      expect(result).to include('def configure(timeout: 30, retries: 3)')
      expect(result).not_to match(/timeout:\s+30\s+retries:/)
      expect(result).to include('@timeout = timeout')
    end

    it 'formats method with parameters without parentheses' do
      source = <<~RUBY
        def foo bar, baz
          puts bar
        end
      RUBY

      result = Rfmt.format(source)

      expect(result).to include('def foo bar, baz')
      expect(result).to include('puts bar')
    end

    it 'formats source metadata nodes' do
      source = <<~RUBY
        puts __FILE__
        puts __LINE__
        puts __ENCODING__
      RUBY

      result = Rfmt.format(source)

      expect(result).to include('__FILE__')
      expect(result).to include('__LINE__')
      expect(result).to include('__ENCODING__')
    end

    it 'formats BEGIN and END blocks' do
      source = <<~RUBY
        BEGIN { setup }
        END { teardown }
      RUBY

      result = Rfmt.format(source)

      expect(result).to include('BEGIN { setup }')
      expect(result).to include('END { teardown }')
    end

    it 'formats endless method with preceding comment (Issue #71)' do
      source = <<~RUBY
        class Test
          # comment
          def a = nil
        end
      RUBY

      # Should not panic (the main fix for Issue #71)
      result = Rfmt.format(source)

      expect(result).to include('class Test')
      expect(result).to include('# comment')
      expect(result).to include('def a')
      expect(result).to include('nil')
    end

    it 'formats endless method without comment' do
      source = <<~RUBY
        class Foo
          def bar = 42
        end
      RUBY

      # Should not panic
      result = Rfmt.format(source)

      expect(result).to include('def bar')
      expect(result).to include('42')
    end

    describe 'inline comments after blocks' do
      it 'preserves inline comments after inline brace blocks' do
        source = "b.each { p it } # c\n"
        expect(Rfmt.format(source)).to eq("b.each { p it } # c\n")
      end

      it 'preserves inline comments after multiline brace blocks' do
        source = <<~RUBY
          b.each {
            p it
          } # c
        RUBY
        expect(Rfmt.format(source)).to include('} # c')
      end

      it 'preserves inline comments with map blocks' do
        source = "[1, 2].map { |x| x * 2 } # double\n"
        expect(Rfmt.format(source)).to eq("[1, 2].map { |x| x * 2 } # double\n")
      end
    end
  end

  describe '.version_info' do
    it 'returns version information' do
      version = Rfmt.version_info

      expect(version).to be_a(String)
      expect(version).to include('Ruby:')
      expect(version).to include('Rust:')
    end
  end
end
