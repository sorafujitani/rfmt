# frozen_string_literal: true

require 'spec_helper'

RSpec.describe Rfmt, 'Rails idioms' do
  def idempotent(source)
    first = Rfmt.format(source)
    second = Rfmt.format(first)
    expect(second).to eq(first)
    first
  end

  describe 'single-line constructs (C fix)' do
    it 'preserves endless methods' do
      source = <<~RUBY
        class Thing
          def active? = status == "active"
          def self.build(x) = new(x)
          def with_default(x = 1) = x + 1
        end
      RUBY
      expect(idempotent(source)).to eq(source)
    end

    it 'preserves inline `def foo; body; end`' do
      source = <<~RUBY
        class A
          def simple; 42; end
          def noop; end
          def guard(x); return if x.nil?; x * 2; end
        end
      RUBY
      expect(idempotent(source)).to eq(source)
    end

    it 'preserves `class Foo < StandardError; end` exception hierarchies' do
      source = <<~RUBY
        module Payments
          class Error < StandardError; end
          class DeclinedError < Error; end
          class GatewayError < Error; end
        end
      RUBY
      expect(idempotent(source)).to eq(source)
    end

    it 'preserves inline modules' do
      source = <<~RUBY
        module Stub; end
      RUBY
      expect(idempotent(source)).to eq(source)
    end
  end

  describe 'blank-line trailing whitespace (A fix)' do
    it 'emits empty blank lines, not indented ones' do
      source = <<~RUBY
        class Foo
          def one
            1
          end

          def two
            2
          end
        end
      RUBY
      formatted = idempotent(source)
      # Each blank line must be strictly empty (no trailing indent spaces).
      formatted.each_line do |line|
        next unless line.strip.empty?

        expect(line).to eq("\n")
      end
    end
  end

  describe 'chain-with-block indentation (D fix)' do
    it 'aligns block body with the chain receiver' do
      source = <<~RUBY
        class Queries
          def run
            User.active
              .where(role: :admin)
              .each do |u|
                puts u.name
              end
          end
        end
      RUBY
      expect(idempotent(source)).to eq(source)
    end
  end

  describe 'heredoc spacing (E fix)' do
    it 'does not insert a blank line before `end` after a heredoc-argument call' do
      source = <<~RUBY
        class A
          def foo
            User.find_by_sql(<<~SQL)
              SELECT *
            SQL
          end
        end
      RUBY
      expect(idempotent(source)).to eq(source)
    end

    it 'preserves a blank separator between heredoc-valued constants' do
      source = <<~RUBY
        class X
          A = <<~T
            body_a
          T

          B = <<~T
            body_b
          T
        end
      RUBY
      expect(idempotent(source)).to eq(source)
    end
  end

  describe 'comments in statement gaps (B2 fix)' do
    it 'does not inflate blank lines when a comment fills the gap between statements' do
      source = <<~RUBY
        class C
          def first
            1
          end
          # second annotation
          def second
            2
          end
        end
      RUBY
      expect(idempotent(source)).to eq(source)
    end

    it 'does not insert a blank line between a trailing-comment line and the next leading comment' do
      source = <<~RUBY
        class H
          def greet(name)
            hello = "hi \#{name}" # inline comment
            # after line
            hello
          end
        end
      RUBY
      expect(idempotent(source)).to eq(source)
    end

    it 'does not insert a blank line between a body-end trailing comment and `end`' do
      source = <<~RUBY
        class C
          def foo
            work!
          rescue => e
            handle(e)
            # end of rescue
          end
        end
      RUBY
      expect(idempotent(source)).to eq(source)
    end
  end

  describe 'block comments =begin/=end (B3 partial fix)' do
    it 'keeps an embdoc attached to the following def inside its class' do
      source = <<~RUBY
        class A
        =begin
        annotation
        =end
          def a
            1
          end
        end
      RUBY
      formatted = idempotent(source)
      # The =begin line must stay at column 0 and remain *inside* class A,
      # before `def a`.
      lines = formatted.split("\n")
      class_idx = lines.index('class A')
      end_idx = lines.index('end')
      begin_idx = lines.index('=begin')
      def_idx = lines.index { |l| l.include?('def a') }

      expect(class_idx).not_to be_nil
      expect(end_idx).not_to be_nil
      expect(begin_idx).not_to be_nil
      expect(def_idx).not_to be_nil
      expect(class_idx < begin_idx).to be true
      expect(begin_idx < def_idx).to be true
      expect(def_idx < end_idx).to be true
    end
  end
end
