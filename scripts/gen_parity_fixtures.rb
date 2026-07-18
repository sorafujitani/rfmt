#!/usr/bin/env ruby
# frozen_string_literal: true

# Regenerates the JSON side of the native-conversion parity fixtures
# (ext/rfmt/tests/fixtures/parity/*.rb -> *.json) through the existing
# Ruby PrismBridge path. Idempotent: output depends only on the .rb sources.

require_relative '../lib/rfmt/prism_bridge'

# Parity means same-version parity: the JSON side must come from the same
# prism line as the ruby-prism crate in ext/rfmt/Cargo.toml.
EXPECTED_PRISM = '1.9'
unless Prism::VERSION.start_with?("#{EXPECTED_PRISM}.")
  abort "prism gem is #{Prism::VERSION}, expected #{EXPECTED_PRISM}.x " \
        '(run via bundle exec; keep Gemfile in sync with the ruby-prism crate)'
end

fixtures_dir = File.expand_path('../ext/rfmt/tests/fixtures/parity', __dir__)
sources = Dir.glob(File.join(fixtures_dir, '*.rb'))
abort "No fixtures found in #{fixtures_dir}" if sources.empty?

sources.each do |path|
  json = Rfmt::PrismBridge.parse(File.read(path))
  File.write(path.sub(/\.rb\z/, '.json'), "#{json}\n")
  puts "generated #{File.basename(path, '.rb')}.json"
end
