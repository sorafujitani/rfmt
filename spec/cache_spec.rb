# frozen_string_literal: true

require 'spec_helper'
require 'rfmt/cache'
require 'rfmt/cli'
require 'tempfile'
require 'fileutils'
require 'digest'

RSpec.describe Rfmt::Cache do
  let(:temp_dir) { Dir.mktmpdir }
  let(:cache_dir) { File.join(temp_dir, 'cache') }
  let(:cache) { described_class.new(cache_dir: cache_dir) }
  let(:test_file) { Tempfile.new(['test', '.rb'], temp_dir) }

  let(:test_code) do
    <<~RUBY
      class Foo
        def bar
          42
        end
      end
    RUBY
  end

  before do
    test_file.write(test_code)
    test_file.close
  end

  after do
    test_file.unlink if test_file.path && File.exist?(test_file.path)
    FileUtils.rm_rf(temp_dir)
  end

  describe '#initialize' do
    it 'creates cache directory if it does not exist' do
      # Cache is initialized in let block, so directory should exist
      expect(Dir.exist?(cache.cache_dir)).to be true
    end

    it 'loads existing cache data' do
      cache.mark_formatted(test_file.path)
      cache.save

      new_cache = described_class.new(cache_dir: cache_dir)
      expect(new_cache.needs_formatting?(test_file.path)).to be false
    end

    it 'handles missing cache file gracefully' do
      expect { described_class.new(cache_dir: cache_dir) }.not_to raise_error
    end

    it 'handles corrupted cache file gracefully' do
      FileUtils.mkdir_p(cache_dir)
      File.write(File.join(cache_dir, 'cache.json'), 'invalid json{')

      expect { described_class.new(cache_dir: cache_dir) }.not_to raise_error
    end
  end

  describe '#needs_formatting?' do
    it 'returns true for files not in cache' do
      expect(cache.needs_formatting?(test_file.path)).to be true
    end

    it 'returns false for files in cache with same hash' do
      cache.mark_formatted(test_file.path)
      expect(cache.needs_formatting?(test_file.path)).to be false
    end

    it 'returns true for files in cache with different hash' do
      cache.mark_formatted(test_file.path)

      # Modify file
      File.write(test_file.path, "#{test_code}\n# comment")

      expect(cache.needs_formatting?(test_file.path)).to be true
    end

    it 'returns true for non-existent files' do
      expect(cache.needs_formatting?('/nonexistent/file.rb')).to be true
    end
  end

  describe '#mark_formatted' do
    it 'adds file to cache with hash and timestamp' do
      cache.mark_formatted(test_file.path)

      expect(cache.needs_formatting?(test_file.path)).to be false
    end

    it 'updates existing cache entry' do
      cache.mark_formatted(test_file.path)
      original_hash = Digest::SHA256.hexdigest(File.read(test_file.path))

      # Modify file
      File.write(test_file.path, "#{test_code}\n# comment")
      cache.mark_formatted(test_file.path)
      new_hash = Digest::SHA256.hexdigest(File.read(test_file.path))

      expect(original_hash).not_to eq(new_hash)
      expect(cache.needs_formatting?(test_file.path)).to be false
    end

    it 'does not mark non-existent files' do
      cache.mark_formatted('/nonexistent/file.rb')
      expect(cache.needs_formatting?('/nonexistent/file.rb')).to be true
    end
  end

  describe '#save' do
    it 'persists cache data to disk' do
      cache.mark_formatted(test_file.path)
      cache.save

      cache_file = File.join(cache_dir, 'cache.json')
      expect(File.exist?(cache_file)).to be true

      data = JSON.parse(File.read(cache_file))
      expect(data).to have_key(test_file.path)
      expect(data[test_file.path]).to have_key('hash')
      expect(data[test_file.path]).to have_key('formatted_at')
      expect(data[test_file.path]).to have_key('version')
    end

    it 'creates valid JSON' do
      cache.mark_formatted(test_file.path)
      cache.save

      cache_file = File.join(cache_dir, 'cache.json')
      expect { JSON.parse(File.read(cache_file)) }.not_to raise_error
    end
  end

  describe '#clear' do
    it 'removes all cache entries' do
      cache.mark_formatted(test_file.path)
      cache.save

      cache.clear

      expect(cache.needs_formatting?(test_file.path)).to be true
    end

    it 'saves empty cache to disk' do
      cache.mark_formatted(test_file.path)
      cache.save
      cache.clear

      cache_file = File.join(cache_dir, 'cache.json')
      data = JSON.parse(File.read(cache_file))
      expect(data).to be_empty
    end
  end

  describe '#invalidate' do
    it 'removes specific file from cache' do
      file1 = test_file
      file2 = Tempfile.new(['test2', '.rb'], temp_dir)
      file2.write(test_code)
      file2.close

      cache.mark_formatted(file1.path)
      cache.mark_formatted(file2.path)

      cache.invalidate(file1.path)

      expect(cache.needs_formatting?(file1.path)).to be true
      expect(cache.needs_formatting?(file2.path)).to be false

      file2.unlink
    end
  end

  describe '#stats' do
    it 'returns cache statistics' do
      cache.mark_formatted(test_file.path)
      cache.save

      stats = cache.stats

      expect(stats).to have_key(:total_files)
      expect(stats).to have_key(:cache_dir)
      expect(stats).to have_key(:cache_size_bytes)
      expect(stats[:total_files]).to eq(1)
      expect(stats[:cache_dir]).to eq(cache_dir)
      expect(stats[:cache_size_bytes]).to be > 0
    end

    it 'returns zero size for missing cache file' do
      stats = cache.stats
      expect(stats[:cache_size_bytes]).to eq(0)
    end
  end

  describe '#prune' do
    it 'removes cache entries for deleted files' do
      file1 = test_file
      file2 = Tempfile.new(['test2', '.rb'], temp_dir)
      file2.write(test_code)
      file2.close

      cache.mark_formatted(file1.path)
      cache.mark_formatted(file2.path)
      cache.save

      # Delete one file
      file2_path = file2.path
      file2.unlink

      pruned = cache.prune

      expect(pruned).to eq(1)
      expect(cache.needs_formatting?(file1.path)).to be false
      expect(cache.needs_formatting?(file2_path)).to be true
    end

    it 'returns zero when no pruning needed' do
      cache.mark_formatted(test_file.path)
      cache.save

      pruned = cache.prune

      expect(pruned).to eq(0)
    end

    it 'saves cache after pruning' do
      file1 = test_file
      file2 = Tempfile.new(['test2', '.rb'], temp_dir)
      file2.write(test_code)
      file2.close

      cache.mark_formatted(file1.path)
      cache.mark_formatted(file2.path)
      cache.save

      file2.unlink

      cache.prune

      # Load new cache instance to verify persistence
      new_cache = described_class.new(cache_dir: cache_dir)
      stats = new_cache.stats
      expect(stats[:total_files]).to eq(1)
    end
  end
end

RSpec.describe 'Cache Integration with CLI' do
  let(:cli) { Rfmt::CLI.new }
  let(:temp_dir) { Dir.mktmpdir }
  let(:cache_dir) { File.join(temp_dir, 'cache') }
  let(:test_file) { Tempfile.new(['test', '.rb'], temp_dir) }

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

    test_file.write(unformatted_code)
    test_file.close
  end

  after do
    test_file.unlink if test_file.path && File.exist?(test_file.path)
    FileUtils.rm_rf(temp_dir)
  end

  describe 'format with cache' do
    it 'formats file on first run' do
      cli.options = { write: true, cache: true, cache_dir: cache_dir }

      cli.format(test_file.path)

      # File should be formatted
      content = File.read(test_file.path)
      expect(content).not_to be_empty
    end

    it 'skips unchanged files on subsequent runs' do
      cli.options = { write: true, cache: true, cache_dir: cache_dir, verbose: true }

      # First run
      cli.format(test_file.path)
      original_content = File.read(test_file.path)

      # Second run (should skip)
      cli.format(test_file.path)
      cached_content = File.read(test_file.path)

      expect(cached_content).to eq(original_content)
    end

    it 'formats files with changed content' do
      cli.options = { write: true, cache: true, cache_dir: cache_dir }

      # First run
      cli.format(test_file.path)
      first_content = File.read(test_file.path)

      # Modify file with unformatted code
      File.write(test_file.path, "class Bar\ndef baz\n123\nend\nend")

      # Second run (should format because content changed)
      cli.format(test_file.path)

      content = File.read(test_file.path)
      # Should be formatted (not equal to unformatted input)
      expect(content).not_to eq("class Bar\ndef baz\n123\nend\nend")
      # Should contain the class
      expect(content).to include('class Bar')
    end

    it 'respects --no-cache option' do
      cli.options = { write: true, cache: false }

      # First run
      cli.format(test_file.path)

      # Second run (should still process without cache)
      expect { cli.format(test_file.path) }.not_to raise_error
    end
  end

  describe 'cache commands' do
    let(:cache_cli) { Rfmt::CacheCommands.new }

    before do
      # Create some cache data
      cache = Rfmt::Cache.new(cache_dir: cache_dir)
      cache.mark_formatted(test_file.path)
      cache.save
    end

    it 'clears cache' do
      cache_cli.options = { cache_dir: cache_dir }
      cache_cli.clear

      cache = Rfmt::Cache.new(cache_dir: cache_dir)
      stats = cache.stats
      expect(stats[:total_files]).to eq(0)
    end

    it 'shows cache stats' do
      cache_cli.options = { cache_dir: cache_dir }
      expect { cache_cli.stats }.not_to raise_error
    end

    it 'prunes stale entries' do
      # Delete the file
      test_file.unlink

      cache_cli.options = { cache_dir: cache_dir }
      cache_cli.prune

      cache = Rfmt::Cache.new(cache_dir: cache_dir)
      stats = cache.stats
      expect(stats[:total_files]).to eq(0)
    end
  end
end
