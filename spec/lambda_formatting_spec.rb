# frozen_string_literal: true

require 'spec_helper'

RSpec.describe Rfmt, 'Lambda Formatting' do
  describe 'simple lambda' do
    it 'formats stabby lambda' do
      source = '-> { 42 }'
      result = Rfmt.format(source)
      expect(result.strip).to eq('-> { 42 }')
    end

    it 'formats lambda with no body' do
      source = '-> {}'
      result = Rfmt.format(source)
      expect(result).to include('->')
    end
  end

  describe 'lambda with parameters' do
    it 'formats single parameter' do
      source = '->(x) { x * 2 }'
      result = Rfmt.format(source)
      expect(result).to include('->(x)')
    end

    it 'formats multiple parameters' do
      source = '->(x, y) { x + y }'
      result = Rfmt.format(source)
      expect(result).to include('->(x, y)')
    end

    it 'formats with default parameter' do
      source = '->(x, y = 10) { x + y }'
      result = Rfmt.format(source)
      expect(result).to include('y = 10')
    end
  end

  describe 'multiline lambda' do
    it 'formats do-end lambda' do
      source = "->(x) do\nx * 2\nend"
      result = Rfmt.format(source)
      expect(result).to include('->(x) do')
      expect(result).to include('end')
    end
  end

  describe 'Rails scope usage' do
    it 'formats scope with lambda' do
      source = 'scope :active, -> { where(active: true) }'
      result = Rfmt.format(source)
      expect(result).to include('-> { where(active: true) }')
    end

    it 'formats scope with parameterized lambda' do
      source = "scope :recent, ->(days) { where('created_at > ?', days.days.ago) }"
      result = Rfmt.format(source)
      expect(result).to include('->(days)')
    end
  end

  describe 'lambda in assignment' do
    it 'formats assigned lambda' do
      source = 'validator = ->(x) { x > 0 }'
      result = Rfmt.format(source)
      expect(result).to include('validator = ->(x)')
    end
  end
end
