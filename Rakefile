# frozen_string_literal: true

require 'bundler/gem_tasks'
require 'rspec/core/rake_task'

RSpec::Core::RakeTask.new(:spec)

require 'rubocop/rake_task'

RuboCop::RakeTask.new

require 'rb_sys/extensiontask'

task build: :compile

GEMSPEC = Gem::Specification.load('rfmt.gemspec')

RbSys::ExtensionTask.new('rfmt', GEMSPEC) do |ext|
  ext.lib_dir = 'lib/rfmt'
end

task default: %i[compile spec rubocop]

# Development tasks
namespace :dev do
  desc 'Run all tests (Ruby + Rust)'
  task :test_all do
    puts 'Running Ruby tests...'
    Rake::Task['spec'].invoke

    puts "\nRunning Rust tests..."
    system('cargo test --manifest-path ext/rfmt/Cargo.toml') || abort('Rust tests failed')
  end

  desc 'Run Rust tests only'
  task :test_rust do
    system('cargo test --manifest-path ext/rfmt/Cargo.toml') || abort('Rust tests failed')
  end

  desc 'Run Rust tests with output'
  task :test_rust_verbose do
    system('cargo test --manifest-path ext/rfmt/Cargo.toml -- --nocapture') || abort('Rust tests failed')
  end

  desc 'Check Rust code with clippy'
  task :clippy do
    system('cargo clippy --manifest-path ext/rfmt/Cargo.toml -- -D warnings') || abort('Clippy found issues')
  end

  desc 'Format Rust code'
  task :fmt_rust do
    system('cargo fmt --manifest-path ext/rfmt/Cargo.toml')
  end

  desc 'Check Rust formatting'
  task :fmt_rust_check do
    system('cargo fmt --manifest-path ext/rfmt/Cargo.toml -- --check') || abort('Rust code needs formatting')
  end

  desc 'Build Rust extension in release mode'
  task :build_release do
    system('cargo build --manifest-path ext/rfmt/Cargo.toml --release') || abort('Release build failed')
  end

  desc 'Clean all build artifacts'
  task :clean do
    system('cargo clean --manifest-path ext/rfmt/Cargo.toml')
    FileUtils.rm_rf('lib/rfmt/rfmt.bundle')
    FileUtils.rm_rf('tmp')
  end

  desc 'Run benchmarks'
  task :bench do
    puts 'Running benchmarks...'
    system('bundle exec rspec spec --tag benchmark') || abort('Benchmarks failed')
  end

  desc 'Show project statistics'
  task :stats do
    puts "\n=== Project Statistics ==="
    puts "\nRuby files:"
    system("find lib spec -name '*.rb' | xargs wc -l | tail -1")
    puts "\nRust files:"
    system("find ext/rfmt/src -name '*.rs' | xargs wc -l | tail -1")
    puts "\nTests:"
    ruby_tests = `grep -r "it\\|describe\\|context" spec --include="*.rb" | wc -l`.strip
    rust_tests = `grep -r "#\\[test\\]" ext/rfmt/src --include="*.rs" | wc -l`.strip
    puts "  Ruby tests: #{ruby_tests}"
    puts "  Rust tests: #{rust_tests}"
  end

  desc 'Interactive development console'
  task :console do
    require 'irb'
    require 'rfmt'
    ARGV.clear
    IRB.start
  end

  desc 'Check node type coverage against Ruby files'
  task :node_coverage, [:dir] do |_t, args|
    dir = args[:dir] || 'lib/'
    system("ruby scripts/check_node_coverage.rb #{dir}")
  end
end

# Aliases for common tasks
desc 'Alias for dev:test_all'
task test_all: 'dev:test_all'

desc 'Alias for dev:console'
task console: 'dev:console'
