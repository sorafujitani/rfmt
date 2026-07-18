# frozen_string_literal: true

require 'spec_helper'

RSpec.describe Rfmt, '.format output validation' do
  context 'when the formatter produces invalid Ruby' do
    before do
      allow(described_class).to receive(:format_code).and_return('def broken(')
    end

    it 'raises Rfmt::Error mentioning invalid output' do
      expect do
        described_class.format('x = 1')
      end.to raise_error(Rfmt::Error, /invalid output/)
    end
  end

  context 'without stubbing' do
    it 'formats valid code normally' do
      result = described_class.format('x = 1')

      expect(result).to include('x = 1')
    end
  end
end
