#!/usr/bin/env ruby
# frozen_string_literal: true

# Regenerates the JSON side of the native-conversion parity fixtures
# (ext/kenshin/tests/fixtures/parity/*.rb -> *.json). The committed .json files
# are frozen golden trees; regenerate only when a fixture .rb changes or a
# new one is added, never to paper over a native_parity.rs failure.
#
# The generating code (Rfmt::PrismBridge) was deleted from lib/ in phase 7 of
# the prism migration; this script loads it from the last git commit that
# still contains it. Needs the development-group prism gem (run via
# bundle exec).

require 'English'
require 'prism'
require 'tmpdir'

# Parity means same-version parity: the JSON side must come from the same
# prism line as the ruby-prism crate in ext/kenshin/Cargo.toml.
EXPECTED_PRISM = '1.9'
unless Prism::VERSION.start_with?("#{EXPECTED_PRISM}.")
  abort "prism gem is #{Prism::VERSION}, expected #{EXPECTED_PRISM}.x " \
        '(run via bundle exec; keep Gemfile in sync with the ruby-prism crate)'
end

REPO = File.expand_path('..', __dir__)
# Git-history paths and the Rfmt module name predate the kenshin rename; the
# historical file defines module Rfmt itself, so no alias is involved here.
BRIDGE_FILES = %w[lib/rfmt/prism_bridge.rb lib/rfmt/prism_node_extractor.rb].freeze

def load_bridge_from_git_history
  # rev-list also returns the deletion commit, so keep the newest commit
  # where the file is still present.
  commits = `git -C "#{REPO}" rev-list HEAD -- #{BRIDGE_FILES.first}`
  abort 'git rev-list failed' unless $CHILD_STATUS.success?

  sha = commits.split.find do |commit|
    system('git', '-C', REPO, 'cat-file', '-e', "#{commit}:#{BRIDGE_FILES.first}", err: File::NULL)
  end
  abort "No commit containing #{BRIDGE_FILES.first} found" if sha.nil?

  Dir.mktmpdir do |dir|
    BRIDGE_FILES.each do |path|
      source = `git -C "#{REPO}" show #{sha}:#{path}`
      abort "Failed to read #{path} at #{sha}" unless $CHILD_STATUS.success?
      File.write(File.join(dir, File.basename(path)), source)
    end
    require File.join(dir, 'prism_bridge')
  end
end

load_bridge_from_git_history

fixtures_dir = File.expand_path('../ext/kenshin/tests/fixtures/parity', __dir__)
sources = Dir.glob(File.join(fixtures_dir, '*.rb'))
abort "No fixtures found in #{fixtures_dir}" if sources.empty?

sources.each do |path|
  json = Rfmt::PrismBridge.parse(File.read(path))
  File.write(path.sub(/\.rb\z/, '.json'), "#{json}\n")
  puts "generated #{File.basename(path, '.rb')}.json"
end
