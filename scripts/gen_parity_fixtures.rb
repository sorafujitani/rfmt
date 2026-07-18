#!/usr/bin/env ruby
# frozen_string_literal: true

# Regenerates the JSON side of the native-conversion parity fixtures
# (ext/rfmt/tests/fixtures/parity/*.rb -> *.json) through the existing
# Ruby PrismBridge path. Idempotent: output depends only on the .rb sources.

require_relative '../lib/rfmt/prism_bridge'

fixtures_dir = File.expand_path('../ext/rfmt/tests/fixtures/parity', __dir__)
sources = Dir.glob(File.join(fixtures_dir, '*.rb'))
abort "No fixtures found in #{fixtures_dir}" if sources.empty?

sources.each do |path|
  json = Rfmt::PrismBridge.parse(File.read(path))
  File.write(path.sub(/\.rb\z/, '.json'), "#{json}\n")
  puts "generated #{File.basename(path, '.rb')}.json"
end
