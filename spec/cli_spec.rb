# frozen_string_literal: true

require 'spec_helper'
require 'rfmt/cli'

RSpec.describe Rfmt::CLI do
  let(:cli) { described_class.new }

  before do
    allow($stdout).to receive(:write)
    allow($stderr).to receive(:write)
  end

  describe '#version' do
    it 'displays version information' do
      expect { cli.version }.not_to raise_error
    end
  end

  describe '#format' do
    it 'formats a Ruby file' do
      require 'tempfile'

      Tempfile.create(['test', '.rb']) do |file|
        file.write("class Foo\ndef bar\n42\nend\nend")
        file.close

        expect { cli.format([file.path]) }.not_to raise_error
      end
    end
  end
end
