# frozen_string_literal: true

require 'spec_helper'

RSpec.describe Kenshin, 'Loop Formatting' do
  it 'formats while loop' do
    source = "while x < 10\nx += 1\nend"
    result = Kenshin.format(source)
    expect(result).to include('while x < 10')
    expect(result).to include('x += 1')
    expect(result).to include('end')
  end

  it 'formats until loop' do
    source = "until done\nwork\nend"
    result = Kenshin.format(source)
    expect(result).to include('until done')
    expect(result).to include('end')
  end

  it 'formats for loop' do
    source = "for i in 1..10\nputs i\nend"
    result = Kenshin.format(source)
    expect(result).to include('for i in 1..10')
    expect(result).to include('end')
  end

  it 'formats break/next in loop' do
    source = "loop do\nbreak if done?\nnext if skip?\nprocess\nend"
    result = Kenshin.format(source)
    expect(result).to include('break if done?')
    expect(result).to include('next if skip?')
  end
end
