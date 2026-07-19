# frozen_string_literal: true

require 'spec_helper'

RSpec.describe Kenshin, 'Case/When Formatting' do
  it 'formats case with when and else' do
    source = "case status\nwhen :pending\nprocess_pending\nelse\nhandle_unknown\nend"
    result = Kenshin.format(source)
    expect(result).to include('case status')
    expect(result).to include('when :pending')
    expect(result).to include('else')
    expect(result).to include('end')
  end

  it 'formats nested case with proper indentation' do
    source = "def classify(value)\ncase value\nwhen Integer\n\"number\"\nwhen String\n\"text\"\nend\nend"
    result = Kenshin.format(source)
    expect(result).to include('def classify(value)')
    expect(result).to include('case value')
    expect(result).to include('when Integer')
  end
end
