# frozen_string_literal: true

require 'spec_helper'

RSpec.describe Rfmt, 'Rails idioms — round 2' do
  def idempotent(source)
    first = Rfmt.format(source)
    second = Rfmt.format(first)
    expect(second).to eq(first), "non-idempotent output:\n#{first}"
    first
  end

  def parses_cleanly?(code)
    require 'prism'
    Prism.parse(code).errors.none?
  end

  describe 'else/ensure clause overshoot (bugs #4/#5)' do
    it 'does not duplicate `end` when begin/rescue/else is followed by end' do
      source = <<~RUBY
        def example
          begin
            body
          rescue => e
            handle(e)
          else
            success
          end
        end
      RUBY
      formatted = idempotent(source)
      expect(formatted.scan(/^\s*end$/m).size).to eq(2)
      expect(parses_cleanly?(formatted)).to be true
    end

    it 'does not duplicate `ensure` when begin has both else and ensure' do
      source = <<~RUBY
        def example
          begin
            body
          rescue => e
            handle(e)
          else
            success
          ensure
            cleanup
          end
        end
      RUBY
      formatted = idempotent(source)
      expect(formatted.scan(/\bensure\b/).size).to eq(1)
      expect(parses_cleanly?(formatted)).to be true
    end
  end

  describe 'heredoc + postfix modifier (bug #3)' do
    it 'keeps the `if` modifier on the opener line, not the terminator line' do
      source = <<~RUBY
        def example(params)
          scope = []
          scope.push(<<~SQL, type: params[:type]) if params[:type]
            SELECT 1
          SQL
          scope
        end
      RUBY
      formatted = idempotent(source)
      # The opener line must carry `if params[:type]`; the SQL terminator
      # must sit alone (leading whitespace only) so Ruby recognizes it.
      expect(formatted).to include('<<~SQL, type: params[:type]) if params[:type]')
      expect(formatted).to match(/^\s*SQL$/m)
      expect(parses_cleanly?(formatted)).to be true
    end
  end

  describe 'multi-line def header with inline comments (bug #2)' do
    it 'preserves inline comments on each parameter line idempotently' do
      source = <<~RUBY
        class C
          def demo(a, # first
                   b, # second
                   c) # third
            [a, b, c]
          end
        end
      RUBY
      formatted = idempotent(source)
      # Each comment should appear exactly once.
      %w[first second third].each do |suffix|
        expect(formatted.scan(/# #{suffix}/).size).to eq(1), "expected # #{suffix} once, got:\n#{formatted}"
      end
      expect(parses_cleanly?(formatted)).to be true
    end
  end

  describe '`x = begin … end` inline (bug #6)' do
    it 'keeps the begin keyword on the same line as the assignment' do
      source = <<~RUBY
        class C
          def demo
            x = begin
              1
            rescue StandardError
              2
            end
            x
          end
        end
      RUBY
      formatted = idempotent(source)
      expect(formatted).to include('    x = begin')
      expect(parses_cleanly?(formatted)).to be true
    end
  end

  describe 'chain reformat respects block scope (bug #9)' do
    it 'does not collapse chain continuations inside a brace lambda' do
      source = <<~RUBY
        class C
          scope :active_in, ->(period) {
            where(active: true)
              .where(last_seen_at: period)
          }
        end
      RUBY
      formatted = idempotent(source)
      # The inner `.where(last_seen_at: …)` must remain indented relative
      # to the receiver `where(active: true)` inside the lambda body.
      expect(formatted).to include("    where(active: true)\n      .where(last_seen_at: period)")
    end

    it 'does not collapse chain continuations inside a do…end block' do
      source = <<~RUBY
        foo.each do |x|
          transform(x)
            .other
        end
      RUBY
      formatted = idempotent(source)
      expect(formatted).to include("  transform(x)\n    .other")
    end
  end

  describe 'heredoc in `if` predicate (bug #10)' do
    it 'does not insert a blank line after the heredoc terminator' do
      source = <<~RUBY
        def demo
          if (sql = <<~SQL)
            SELECT 1
          SQL
            execute(sql)
          end
        end
      RUBY
      formatted = idempotent(source)
      expect(formatted).to match(/SQL\n    execute/)
    end
  end

  describe 'unary `!@` method name' do
    it 'preserves the `@` suffix on `def !@`' do
      source = <<~RUBY
        class C
          def !@
            false
          end
        end
      RUBY
      formatted = idempotent(source)
      expect(formatted).to include('def !@')
      expect(parses_cleanly?(formatted)).to be true
    end
  end
end
