# frozen_string_literal: true

require 'spec_helper'

RSpec.describe Rfmt::PrismBridge do
  describe '.parse' do
    it 'parses Ruby code and returns JSON' do
      result = described_class.parse("puts 'hello'")

      expect(result).to be_a(String)
      expect { JSON.parse(result) }.not_to raise_error
    end

    it 'raises error for invalid syntax' do
      expect do
        described_class.parse('class Foo def')
      end.to raise_error(Rfmt::PrismBridge::ParseError)
    end
  end
end
