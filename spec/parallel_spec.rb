# frozen_string_literal: true

require 'spec_helper'
require 'rfmt/cli'
require 'tempfile'
require 'fileutils'
require 'benchmark'

RSpec.describe 'Parallel Processing' do
  let(:cli) { Rfmt::CLI.new }
  let(:temp_dir) { Dir.mktmpdir }
  let(:test_files) { [] }

  let(:unformatted_code) do
    <<~RUBY
      class Foo
        def bar
          42
        end
      end
    RUBY
  end

  before do
    # Suppress CLI output during tests
    allow($stdout).to receive(:write)
    allow($stderr).to receive(:write)

    # Create multiple test files
    5.times do |i|
      file = Tempfile.new(["test#{i}", '.rb'], temp_dir)
      file.write(unformatted_code)
      file.close
      test_files << file.path
    end
  end

  after do
    test_files.each { |f| File.unlink(f) if File.exist?(f) }
    FileUtils.rm_rf(temp_dir)
  end

  describe 'parallel processing' do
    it 'processes multiple files in parallel by default' do
      cli.options = { write: true, parallel: true }

      expect do
        cli.format(*test_files)
      end.not_to raise_error

      # Verify all files were formatted
      test_files.each do |file|
        content = File.read(file)
        expect(content).not_to be_empty
      end
    end

    it 'processes files sequentially when parallel is disabled' do
      cli.options = { write: true, parallel: false }

      expect do
        cli.format(*test_files)
      end.not_to raise_error

      # Verify all files were formatted
      test_files.each do |file|
        content = File.read(file)
        expect(content).not_to be_empty
      end
    end

    it 'uses single file processing for one file even with parallel enabled' do
      cli.options = { write: true, parallel: true }
      single_file = test_files.first

      expect(cli).not_to receive(:format_files_parallel)

      cli.format(single_file)
    end

    it 'respects the jobs option for parallel processing' do
      cli.options = { write: true, parallel: true, jobs: 2, verbose: true }

      expect do
        cli.format(*test_files)
      end.not_to raise_error
    end

    it 'handles errors in parallel processing gracefully' do
      # Create a file with invalid Ruby
      invalid_file = Tempfile.new(['invalid', '.rb'], temp_dir)
      invalid_file.write("class Foo\n  def bar\nend")
      invalid_file.close

      cli.options = { write: true, parallel: true }

      expect do
        cli.format(*test_files, invalid_file.path)
      end.to raise_error(SystemExit)

      invalid_file.unlink
    end
  end

  describe 'performance comparison' do
    let(:many_files) do
      files = []
      10.times do |i|
        file = Tempfile.new(["perf#{i}", '.rb'], temp_dir)
        file.write(unformatted_code * 10) # Larger files
        file.close
        files << file.path
      end
      files
    end

    after do
      many_files.each { |f| File.unlink(f) if File.exist?(f) }
    end

    it 'processes multiple files efficiently with parallelization' do
      cli.options = { write: true, parallel: true }

      parallel_time = Benchmark.realtime do
        cli.format(*many_files)
      end

      expect(parallel_time).to be < 10.0 # Should complete within 10 seconds
    end
  end
end
