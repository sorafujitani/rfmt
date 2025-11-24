# frozen_string_literal: true

require 'mkmf'
require 'rb_sys/mkmf'

# Set the cargo build target to match the system architecture
if RUBY_PLATFORM =~ /arm64-darwin/
  ENV['CARGO_BUILD_TARGET'] = 'aarch64-apple-darwin'
elsif RUBY_PLATFORM =~ /x86_64-darwin/
  ENV['CARGO_BUILD_TARGET'] = 'x86_64-apple-darwin'
end

create_rust_makefile('rfmt/rfmt')
