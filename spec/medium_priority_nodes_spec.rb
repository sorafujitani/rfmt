# frozen_string_literal: true

require 'spec_helper'

RSpec.describe Kenshin, 'Medium Priority Nodes' do
  it 'formats class variable or-assign' do
    source = '@@cache ||= {}'
    result = Kenshin.format(source)
    expect(result).to include('@@cache ||= {}')
  end

  it 'formats global variable' do
    source = '$DEBUG = true'
    result = Kenshin.format(source)
    expect(result).to include('$DEBUG = true')
  end

  it 'formats local variable or-assign (memoization)' do
    source = '@cache ||= {}'
    result = Kenshin.format(source)
    expect(result).to include('@cache ||= {}')
  end

  it 'formats pattern matching (case-in)' do
    source = "case data\nin { name: }\nputs name\nend"
    result = Kenshin.format(source)
    expect(result).to include('case data')
    expect(result).to include('in {')
  end

  it 'formats singleton class' do
    source = "class << self\ndef foo\nend\nend"
    result = Kenshin.format(source)
    expect(result).to include('class << self')
  end

  it 'formats multiple assignment' do
    source = 'a, b, c = [1, 2, 3]'
    result = Kenshin.format(source)
    expect(result).to include('a, b, c = [1, 2, 3]')
  end

  it 'formats block argument' do
    source = 'items.map(&:to_s)'
    result = Kenshin.format(source)
    expect(result).to include('&:to_s')
  end

  it 'formats hash splat' do
    source = 'method(**options)'
    result = Kenshin.format(source)
    expect(result).to include('**options')
  end
end
