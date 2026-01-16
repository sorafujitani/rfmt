# frozen_string_literal: true

require 'spec_helper'
require 'rfmt/native_extension_loader'

RSpec.describe Rfmt::NativeExtensionLoader do
  describe '.load_extension' do
    let(:loader) { described_class }

    before do
      # Reset any previous load attempts
      allow(loader).to receive(:require).and_return(false)
      allow(loader).to receive(:warn) # Suppress debug output in tests
    end

    context 'with Ruby 3.3' do
      before do
        stub_const('RUBY_VERSION', '3.3.0')
      end

      it 'tries version-specific directory first' do
        # Mock successful load from version-specific directory
        allow(loader).to receive(:require).with(anything) do |path|
          path.include?('3.3/rfmt')
        end

        expect(loader.load_extension).to be true
      end

      it 'falls back to direct path if version directory does not exist' do
        # First path fails, second path succeeds
        call_count = 0
        allow(loader).to receive(:require) do
          call_count += 1
          call_count == 2 # Second path succeeds
        end

        expect(loader.load_extension).to be true
      end
    end

    context 'with Ruby 3.0' do
      before do
        stub_const('RUBY_VERSION', '3.0.4')
      end

      it 'tries version directory and falls back to direct path' do
        # Mock successful load from direct path
        allow(loader).to receive(:require).with(anything) do |path|
          path.end_with?('rfmt/rfmt')
        end

        expect(loader.load_extension).to be true
      end
    end

    context 'with Ruby 2.7 (legacy)' do
      before do
        stub_const('RUBY_VERSION', '2.7.8')
      end

      it 'loads from direct path' do
        # Should try direct path first for older Ruby
        allow(loader).to receive(:require).with(anything) do |path|
          path.end_with?('rfmt/rfmt') || path.end_with?('rfmt.bundle')
        end

        expect(loader.load_extension).to be true
      end
    end

    context 'when extension cannot be found' do
      before do
        allow(loader).to receive(:require).and_raise(LoadError, 'cannot load such file')
      end

      it 'raises a detailed LoadError' do
        expect { loader.load_extension }.to raise_error(LoadError) do |error|
          expect(error.message).to include('Unable to load rfmt native extension')
          expect(error.message).to include('Tried the following paths')
          expect(error.message).to include('https://github.com/fs0414/rfmt/issues')
        end
      end
    end

    context 'with debug logging' do
      before do
        stub_const('RUBY_VERSION', '3.3.0')
        ENV['RFMT_DEBUG'] = '1'
        allow(loader).to receive(:warn).and_call_original
      end

      after do
        ENV.delete('RFMT_DEBUG')
      end

      it 'outputs debug information when RFMT_DEBUG is set' do
        allow(loader).to receive(:require).with(anything) do |path|
          path.include?('3.3/rfmt')
        end

        expect(loader).to receive(:warn).with(/Loading native extension for Ruby 3.3.0/)
        expect(loader).to receive(:warn).with(/Trying paths:/)
        expect(loader).to receive(:warn).with(/Successfully loaded/)

        loader.load_extension
      end
    end
  end
end
