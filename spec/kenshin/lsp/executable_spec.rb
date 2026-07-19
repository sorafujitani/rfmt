# frozen_string_literal: true

require 'spec_helper'

RSpec.describe 'kenshin-lsp executable' do
  it 'is packaged as an executable' do
    executable = File.expand_path('../../../exe/kenshin-lsp', __dir__)

    expect(File.file?(executable)).to eq(true)
    expect(File.executable?(executable)).to eq(true)
    expect(Gem::Specification.load('kenshin.gemspec').executables).to include('kenshin-lsp')
  end
end
