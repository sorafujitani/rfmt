# frozen_string_literal: true

require 'spec_helper'

RSpec.describe Rfmt, 'Rescue/Ensure Indentation' do
  def normalize(source)
    source.gsub(/[ \t]+\n/, "\n").chomp
  end

  describe 'def with rescue' do
    it 'keeps rescue aligned with def and body indented one level deeper' do
      source = <<~RUBY
        def baz
          risky!
        rescue
          handle!
          cleanup!
        end
      RUBY
      expected = <<~RUBY.chomp
        def baz
          risky!
        rescue
          handle!
          cleanup!
        end
      RUBY
      expect(normalize(Rfmt.format(source))).to eq(expected)
    end

    it 'keeps rescue aligned with def when nested inside a class' do
      source = <<~RUBY
        module Admins
          class SessionsController
            def create
              authenticate!
              respond_with(resource)
            rescue StandardError
              flash[:alert] = 'nope'
              redirect_to login_path
            end
          end
        end
      RUBY
      expected = <<~RUBY.chomp
        module Admins
          class SessionsController
            def create
              authenticate!
              respond_with(resource)
            rescue StandardError
              flash[:alert] = 'nope'
              redirect_to login_path
            end
          end
        end
      RUBY
      expect(normalize(Rfmt.format(source))).to eq(expected)
    end

    it 'handles chained rescue clauses' do
      source = <<~RUBY
        def foo
          body!
        rescue ArgumentError => e
          handle_arg(e)
        rescue StandardError => e
          handle_std(e)
        end
      RUBY
      expected = <<~RUBY.chomp
        def foo
          body!
        rescue ArgumentError => e
          handle_arg(e)
        rescue StandardError => e
          handle_std(e)
        end
      RUBY
      expect(normalize(Rfmt.format(source))).to eq(expected)
    end

    it 'handles rescue followed by ensure' do
      source = <<~RUBY
        def foo
          work!
        rescue StandardError => e
          log(e)
        ensure
          cleanup!
        end
      RUBY
      expected = <<~RUBY.chomp
        def foo
          work!
        rescue StandardError => e
          log(e)
        ensure
          cleanup!
        end
      RUBY
      expect(normalize(Rfmt.format(source))).to eq(expected)
    end

    it 'handles ensure-only clause' do
      source = <<~RUBY
        def foo
          work!
        ensure
          cleanup!
          log!
        end
      RUBY
      expected = <<~RUBY.chomp
        def foo
          work!
        ensure
          cleanup!
          log!
        end
      RUBY
      expect(normalize(Rfmt.format(source))).to eq(expected)
    end
  end

  describe 'do...end block with rescue' do
    it 'aligns rescue with the block opener and indents the body' do
      source = <<~RUBY
        foo.each do |x|
          transform(x)
        rescue StandardError => e
          log(e)
          raise
        end
      RUBY
      expected = <<~RUBY.chomp
        foo.each do |x|
          transform(x)
        rescue StandardError => e
          log(e)
          raise
        end
      RUBY
      expect(normalize(Rfmt.format(source))).to eq(expected)
    end
  end

  describe 'explicit begin...end' do
    it 'indents begin body and aligns rescue/ensure with begin' do
      source = <<~RUBY
        def outer
          begin
            risky!
            more!
          rescue StandardError
            handle!
          ensure
            cleanup!
          end
        end
      RUBY
      expected = <<~RUBY.chomp
        def outer
          begin
            risky!
            more!
          rescue StandardError
            handle!
          ensure
            cleanup!
          end
        end
      RUBY
      expect(normalize(Rfmt.format(source))).to eq(expected)
    end
  end
end
