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

  describe '--config' do
    require 'tmpdir'

    it 'applies formatter settings from the explicit config file' do
      Dir.mktmpdir do |dir|
        config_path = File.join(dir, 'custom.yml')
        File.write(config_path, "formatting:\n  indent_width: 4\n")
        file = File.join(dir, 'test.rb')
        File.write(file, "class Foo\ndef bar\n42\nend\nend\n")

        described_class.start(['format', '--config', config_path, '--no-cache', file])

        expect(File.read(file).lines).to include("    def bar\n")
      end
    end

    it 'fails fast when the config file does not exist' do
      expect do
        described_class.start(['format', '--config', 'missing.yml', 'x.rb'])
      end.to raise_error(SystemExit)
    end

    it 'fails fast when the config file is invalid' do
      Dir.mktmpdir do |dir|
        config_path = File.join(dir, 'broken.yml')
        File.write(config_path, "formatting:\n  line_length: 20\n")
        file = File.join(dir, 'test.rb')
        File.write(file, "x = 1\n")

        expect do
          described_class.start(['format', '--config', config_path, '--no-cache', file])
        end.to raise_error(SystemExit)
      end
    end
  end

  describe '#config_cmd' do
    it 'shows the effective formatter configuration' do
      Dir.mktmpdir do |dir|
        config_path = File.join(dir, 'custom.yml')
        File.write(config_path, "formatting:\n  indent_width: 7\n")

        output = StringIO.new
        allow($stdout).to receive(:write) { |s| output.write(s) }
        described_class.start(['config_cmd', '--config', config_path])

        expect(output.string).to include('indent_width: 7')
        expect(output.string).to include('line_length: 100')
      end
    end
  end
end
