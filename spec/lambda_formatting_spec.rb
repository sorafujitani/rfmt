# frozen_string_literal: true

require 'spec_helper'

RSpec.describe Kenshin, 'Lambda Formatting' do
  it 'formats stabby lambda' do
    source = '-> { 42 }'
    result = Kenshin.format(source)
    expect(result.strip).to eq('-> { 42 }')
  end

  it 'formats Rails scope with lambda' do
    source = 'scope :active, -> { where(active: true) }'
    result = Kenshin.format(source)
    expect(result).to include('-> { where(active: true) }')
  end
end
