# frozen_string_literal: true

require 'spec_helper'

RSpec.describe Rfmt, 'Node Types' do
  describe 'Symbol nodes' do
    it 'formats symbol literal' do
      source = ':hello'
      result = Rfmt.format(source)
      expect(result.strip).to eq(':hello')
    end

    it 'formats symbol in method call' do
      source = 'foo(:bar)'
      result = Rfmt.format(source)
      expect(result).to include(':bar')
    end

    it 'formats symbol as hash value' do
      source = '{ status: :active }'
      result = Rfmt.format(source)
      expect(result).to include(':active')
    end
  end

  describe 'Variable nodes' do
    it 'formats local variable assignment' do
      source = 'x = 42'
      result = Rfmt.format(source)
      expect(result.strip).to eq('x = 42')
    end

    it 'formats local variable read' do
      source = "x = 1\nputs x"
      result = Rfmt.format(source)
      expect(result).to include('puts x')
    end

    it 'formats instance variable assignment' do
      source = '@user = User.new'
      result = Rfmt.format(source)
      expect(result).to include('@user = User.new')
    end

    it 'formats instance variable read' do
      source = "def name\n  @name\nend"
      result = Rfmt.format(source)
      expect(result).to include('@name')
    end

    it 'formats return statement' do
      source = "def foo\n  return nil if x.nil?\n  x\nend"
      result = Rfmt.format(source)
      expect(result).to include('return nil if x.nil?')
    end

    it 'formats return with value' do
      source = "def bar\n  return 42\nend"
      result = Rfmt.format(source)
      expect(result).to include('return 42')
    end
  end

  describe 'Or node' do
    it 'formats logical or' do
      source = 'x || y'
      result = Rfmt.format(source)
      expect(result.strip).to eq('x || y')
    end

    it 'formats memoization pattern' do
      source = '@cache ||= {}'
      result = Rfmt.format(source)
      expect(result).to include('@cache ||= {}')
    end

    it 'formats or with method call' do
      source = 'name || "Anonymous"'
      result = Rfmt.format(source)
      expect(result).to include('name || "Anonymous"')
    end
  end

  describe 'Hash nodes' do
    it 'formats hash literal' do
      source = '{ a: 1, b: 2 }'
      result = Rfmt.format(source)
      expect(result).to include('a: 1')
      expect(result).to include('b: 2')
    end

    it 'formats keyword arguments' do
      source = 'foo(bar: 1, baz: 2)'
      result = Rfmt.format(source)
      expect(result).to include('bar: 1')
      expect(result).to include('baz: 2')
    end

    it 'formats Rails-style options' do
      source = 'validates :email, presence: true, uniqueness: true'
      result = Rfmt.format(source)
      expect(result).to include('presence: true')
      expect(result).to include('uniqueness: true')
    end

    it 'formats render with options' do
      source = 'render json: @user, status: :ok'
      result = Rfmt.format(source)
      expect(result).to include('json: @user')
      expect(result).to include('status: :ok')
    end
  end

  # rubocop:disable Lint/InterpolationCheck
  describe 'String interpolation' do
    it 'formats simple interpolation' do
      source = '"Hello, #{name}!"'
      result = Rfmt.format(source)
      expect(result).to include('#{name}')
    end

    it 'formats complex interpolation' do
      source = '"Count: #{items.count} items"'
      result = Rfmt.format(source)
      expect(result).to include('#{items.count}')
    end

    it 'formats multiple interpolations' do
      source = '"#{first} and #{second}"'
      result = Rfmt.format(source)
      expect(result).to include('#{first}')
      expect(result).to include('#{second}')
    end

    it 'formats interpolation with method chain' do
      source = '"User: #{user.name.upcase}"'
      result = Rfmt.format(source)
      expect(result).to include('#{user.name.upcase}')
    end
  end
  # rubocop:enable Lint/InterpolationCheck
end
