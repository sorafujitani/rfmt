# frozen_string_literal: true

require 'spec_helper'
require 'uri'
require 'ruby_lsp/rfmt/formatter_runner'

RSpec.describe RubyLsp::Rfmt::FormatterRunner do
  let(:runner) { described_class.new }

  describe '#run_formatting' do
    let(:uri) { URI::Generic.build(path: '/test.rb') }

    context 'with valid Ruby code' do
      it 'returns formatted code' do
        source = 'def foo() end'
        document = double('Document', source: source)

        result = runner.run_formatting(uri, document)

        expect(result).to be_a(String)
        expect(result).not_to be_empty
      end
    end

    context 'with syntax error' do
      it 'returns nil' do
        source = 'def foo('
        document = double('Document', source: source)

        result = runner.run_formatting(uri, document)

        expect(result).to be_nil
      end
    end

    context 'with empty source' do
      it 'returns newline for empty input' do
        source = ''
        document = double('Document', source: source)

        result = runner.run_formatting(uri, document)

        expect(result).to eq("\n")
      end
    end
  end

  describe '#run_diagnostic' do
    it 'returns empty array' do
      uri = URI::Generic.build(path: '/test.rb')
      document = double('Document', source: 'def foo; end')

      result = runner.run_diagnostic(uri, document)

      expect(result).to eq([])
    end
  end
end
