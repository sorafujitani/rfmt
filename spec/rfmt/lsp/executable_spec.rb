# frozen_string_literal: true

require 'spec_helper'

RSpec.describe 'rfmt-lsp executable' do
  it 'is packaged as an executable' do
    executable = File.expand_path('../../../exe/rfmt-lsp', __dir__)

    expect(File.file?(executable)).to eq(true)
    expect(File.executable?(executable)).to eq(true)
    expect(Gem::Specification.load('rfmt.gemspec').executables).to include('rfmt-lsp')
  end
end
