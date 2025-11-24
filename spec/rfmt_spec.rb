# frozen_string_literal: true

require 'spec_helper'

RSpec.describe Rfmt do
  describe '.format' do
    it 'formats simple Ruby code' do
      source = <<~RUBY
        class Foo
        def bar
        42
        end
        end
      RUBY

      result = Rfmt.format(source)

      expect(result).to be_a(String)
      expect(result).to include('class Foo')
      expect(result).to include('def bar')
      expect(result).to include('end')
    end

    it 'raises error for invalid Ruby syntax' do
      invalid_source = 'class Foo def'

      expect {
        Rfmt.format(invalid_source)
      }.to raise_error(Rfmt::Error)
    end
  end

  describe '.version_info' do
    it 'returns version information' do
      version = Rfmt.version_info

      expect(version).to be_a(String)
      expect(version).to include('Ruby:')
      expect(version).to include('Rust:')
    end
  end
end
