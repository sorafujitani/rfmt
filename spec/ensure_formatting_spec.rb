# frozen_string_literal: true

require 'spec_helper'

RSpec.describe Kenshin, 'Ensure Formatting' do
  it 'formats begin-rescue-ensure block' do
    source = "begin\nrisky\nrescue StandardError => e\nhandle(e)\nensure\ncleanup\nend"
    result = Kenshin.format(source)
    expect(result).to include('rescue StandardError => e')
    expect(result).to include('ensure')
    expect(result).to include('cleanup')
  end

  it 'formats method with implicit begin-ensure' do
    source = "def process\nacquire_lock\nensure\nrelease_lock\nend"
    result = Kenshin.format(source)
    expect(result).to include('def process')
    expect(result).to include('ensure')
    expect(result).to include('release_lock')
  end
end
