# Development Guide

This guide covers testing, building, and releasing kenshin.

## Table of Contents

- [Prerequisites](#prerequisites)
- [Building](#building)
- [Testing](#testing)
- [Development Workflow](#development-workflow)
- [Release Process](#release-process)
- [Troubleshooting](#troubleshooting)

## Prerequisites

### Required Tools

- **Ruby**: 3.3 or later
- **Rust**: 1.70 or later (installed via rustup)
- **Bundler**: `gem install bundler`
- **Rake**: Included in Ruby standard library

### System Dependencies

**Install Rust** (via rustup - works on all platforms):
```bash
# Install rustup and Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Follow the on-screen instructions, then:
source $HOME/.cargo/env

# Verify installation
rustc --version
cargo --version
```

**Additional dependencies**:

- **macOS**: Xcode Command Line Tools
  ```bash
  xcode-select --install
  ```

- **Linux (Debian/Ubuntu)**:
  ```bash
  sudo apt-get update
  sudo apt-get install build-essential
  ```

- **Windows**:
  - Download and run rustup-init.exe from https://rustup.rs
  - Install Visual Studio C++ Build Tools

### Setup

```bash
# Clone repository
git clone https://github.com/yourusername/kenshin.git
cd kenshin

# Install Ruby dependencies
bundle install

# Build native extension
bundle exec rake compile
```

## Building

### Clean Build

```bash
# Clean all build artifacts
bundle exec rake clean
bundle exec rake clobber

# Rebuild from scratch
bundle exec rake compile
```

### Development Build

```bash
# Quick rebuild (only changed files)
bundle exec rake compile
```

### Build Options

```bash
# Build in debug mode (faster compilation, slower runtime)
cd ext/kenshin
cargo build

# Build in release mode (default for rake compile)
cargo build --release

# Check for compilation errors without building
cargo check
```

## Testing

### Ruby Tests

#### Run All Tests

```bash
# All RSpec tests
bundle exec rake spec

# or
bundle exec rspec
```

#### Run Specific Tests

```bash
# Single test file
bundle exec rspec spec/formatter_spec.rb

# Specific test by line number
bundle exec rspec spec/formatter_spec.rb:45

# Tests matching a pattern
bundle exec rspec spec/formatter_spec.rb -e "indentation"
```

#### Test Output Options

```bash
# Documentation format (detailed)
bundle exec rspec --format documentation

# Progress format (default)
bundle exec rspec --format progress

# Only failures
bundle exec rspec --only-failures
```

#### CLI Tests ⭐

Test the command-line interface functionality:

```bash
# Run all CLI tests
bundle exec rspec spec/cli_spec.rb

# Run specific CLI test
bundle exec rspec spec/cli_spec.rb -e "format with diff option"
```

**CLI Test Coverage:**
- Version command (`kenshin version`)
- Format command with various options (`--write`, `--no-write`, `--check`, `--diff`, `--verbose`)
- Check mode with proper exit codes (0 for formatted, 1 for needs formatting)
- Diff display in 3 formats (unified, color, side_by_side)
- Multiple file processing
- Error handling (syntax errors, missing files)
- Init command (`.kenshin.yml` creation)
- Config command (configuration display)

**Example Test Cases:**
```ruby
# Test format with write option
it 'formats and writes to file' do
  cli.options = { write: true }
  cli.format(temp_file.path)

  formatted = File.read(temp_file.path)
  expect(formatted).to eq(formatted_code)
end

# Test check mode exit codes
it 'exits with code 1 when formatting is needed' do
  cli.options = { check: true, write: false }

  expect do
    cli.format(temp_file.path)
  end.to raise_error(SystemExit) do |error|
    expect(error.status).to eq(1)
  end
end

# Test diff display
it 'shows unified diff' do
  cli.options = { diff: true, write: false, diff_format: 'unified' }
  expect { cli.format(temp_file.path) }.not_to raise_error
end
```

#### Configuration Tests ⭐

Test the YAML configuration system:

```bash
# Run all configuration tests
bundle exec rspec spec/configuration_spec.rb

# Run specific configuration test
bundle exec rspec spec/configuration_spec.rb -e "discovers .kenshin.yml"
```

**Configuration Test Coverage:**
- Auto-discovery of config files (`.kenshin.yml`, `.kenshin.yaml`, `kenshin.yml`, `kenshin.yaml`)
- Default configuration loading
- Custom configuration file loading
- Configuration merging (deep merge for nested hashes)
- Validation (line_length > 0, indent_width > 0)
- File pattern matching (include/exclude)
- Formatting options retrieval

**Example Test Cases:**
```ruby
# Test config file discovery
it 'discovers .kenshin.yml' do
  File.write('.kenshin.yml', "version: '1.0'")
  config = described_class.discover
  expect(config).to be_a(described_class)
end

# Test configuration validation
it 'validates positive line_length' do
  expect do
    described_class.new('formatting' => { 'line_length' => -1 })
  end.to raise_error(Kenshin::Configuration::ConfigError, 'line_length must be positive')
end

# Test file pattern matching
it 'includes files matching include patterns' do
  config = described_class.new
  files = config.files_to_format(base_path: temp_dir)
  expect(files).to include(File.join(temp_dir, 'lib', 'test.rb'))
end
```

### Rust Tests

#### Run All Rust Tests

```bash
cd ext/kenshin

# All tests
cargo test

# Library tests only (no integration tests)
cargo test --lib

# With output
cargo test -- --nocapture
```

#### Run Specific Rust Tests

```bash
# Tests in a specific module
cargo test ast::tests

# Single test
cargo test test_node_creation

# Tests matching pattern
cargo test parse
```

### Corpus Check

The corpus check formats every Ruby file in the repository and verifies with the prism gem (a development-only dependency) that the reformatted output parses to a structurally identical AST. Together with the parity fixtures it is the main safety net for formatter changes:

```bash
bundle exec ruby scripts/corpus_check.rb
```

### Coverage

```bash
# Install coverage tool (once)
cargo install cargo-tarpaulin

# Generate coverage report
cd ext/kenshin
cargo tarpaulin --out Html --output-dir ../../coverage
```

## Development Workflow

### 1. Make Changes

Edit files in:
- `lib/` - Ruby code
- `ext/kenshin/src/` - Rust code
- `spec/` - Tests

### 2. Build & Test

```bash
# After changing Rust code
bundle exec rake compile

# Run tests
bundle exec rake spec

# Run Rust tests
cd ext/kenshin && cargo test
```

### 3. Verify

```bash
# Manual testing with IRB
bundle exec irb -I lib -r kenshin

# In IRB:
input = "class Foo\nend"
puts Kenshin.format(input)
```

### 4. Format & Lint

```bash
# Format Rust code
cd ext/kenshin
cargo fmt

# Check lints
cargo clippy

# Format Ruby code
bundle exec rubocop -a
```

### 5. Run Full Test Suite

```bash
# All tests
bundle exec rake

# or
bundle exec rake spec
cd ext/kenshin && cargo test
```

## Release Process

### Pre-Release Checklist

- [ ] All tests passing
- [ ] Version updated in `lib/kenshin/version.rb`
- [ ] Version updated in `ext/kenshin/Cargo.toml`
- [ ] CHANGELOG.md updated
- [ ] Documentation updated
- [ ] No uncommitted changes

### Version Update

1. **Update Ruby version** (`lib/kenshin/version.rb`):

```ruby
module Kenshin
  VERSION = "0.2.0"  # Update this
end
```

2. **Update Rust version** (`ext/kenshin/Cargo.toml`):

```toml
[package]
name = "kenshin"
version = "0.2.0"  # Update this
```

3. **Update CHANGELOG.md**:

```markdown
## [0.2.0] - 2025-01-15

### Added
- New feature X
- New feature Y

### Fixed
- Bug fix Z
```

### Build Gem

```bash
# Build gem package
gem build kenshin.gemspec

# This creates: kenshin-0.2.0.gem
```

### Test Gem Locally

```bash
# Install locally
gem install kenshin-0.2.0.gem

# Test it
irb
> require 'kenshin'
> Kenshin.format("class Foo\nend")
```

### Publish to RubyGems

#### First Time Setup

```bash
# Create RubyGems account at https://rubygems.org

# Get API key
curl -u your_username https://rubygems.org/api/v1/api_key.yaml > ~/.gem/credentials
chmod 0600 ~/.gem/credentials
```

#### Push to RubyGems

```bash
# Push the gem
gem push kenshin-0.2.0.gem

# Verify at https://rubygems.org/gems/kenshin
```

### Post-Release

1. **Create Git Tag**:

```bash
git tag -a v0.2.0 -m "Release version 0.2.0"
git push origin v0.2.0
```

2. **Create GitHub Release**:

- Go to https://github.com/yourusername/kenshin/releases/new
- Select tag `v0.2.0`
- Set release title: `v0.2.0`
- Copy CHANGELOG entry to description
- Attach `kenshin-0.2.0.gem` file
- Publish release

3. **Announce**:

- Update README if needed
- Post on Ruby forums/communities if significant release

## Troubleshooting

### Build Issues

#### Problem: "cargo: command not found"

```bash
# Install Rust via rustup (official method)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Add Cargo to PATH for current session
source $HOME/.cargo/env

# To make it permanent, rustup installer adds this line to your shell profile:
# export PATH="$HOME/.cargo/bin:$PATH"

# Verify installation
cargo --version
```

#### Problem: "magnus version mismatch"

```bash
# Clean and rebuild
bundle exec rake clobber
bundle exec rake compile
```

#### Problem: Build fails after git pull

```bash
# Update dependencies
bundle install
cd ext/kenshin && cargo update

# Clean rebuild
bundle exec rake clobber compile
```

### Test Issues

#### Problem: Tests fail after changes

```bash
# Rebuild extension
bundle exec rake compile

# Clear Ruby cache
rm -rf tmp/

# Run tests again
bundle exec rspec
```

#### Problem: "Cannot load such file -- kenshin/kenshin"

```bash
# Extension not built or not in correct location
bundle exec rake compile

# Check if extension exists
ls -la lib/kenshin/kenshin.bundle  # macOS
ls -la lib/kenshin/kenshin.so      # Linux
```

### Runtime Issues

#### Problem: "Prism integration error"

Parsing happens inside the Rust extension via the statically linked ruby-prism crate, so this error indicates a bug in kenshin itself rather than a dependency problem. The prism gem in the Gemfile is a development-only dependency used by the corpus check and parity fixtures; updating it does not affect runtime parsing. Please open an issue with the input that triggers the error.

#### Problem: Segmentation fault

This usually indicates a bug in Rust code. To debug:

```bash
# Build debug version
cd ext/kenshin
cargo build

# Run with debugging
RUST_BACKTRACE=1 bundle exec ruby your_test.rb
```

### Performance Issues

#### Problem: Formatting is slow

```bash
# Make sure using release build
bundle exec rake compile  # Uses --release by default

# Verify
file lib/kenshin/kenshin.bundle
# Should say "not stripped" for debug, "stripped" for release
```

## Development Tips

### Fast Iteration

```bash
# Terminal 1: Watch for file changes
while true; do
  inotifywait -e modify ext/kenshin/src/*.rs
  bundle exec rake compile
done

# Terminal 2: Run tests
bundle exec rspec
```

### Debugging

#### Ruby Side

```ruby
# Add to code
require 'debug'
binding.break  # Ruby 3.1+

# or
require 'pry'
binding.pry
```

#### Rust Side

```rust
// Add to code
dbg!(&some_variable);

// or
eprintln!("Debug: {:?}", some_value);
```

Run with:

```bash
RUST_BACKTRACE=1 bundle exec rspec
```

### Benchmarking

```ruby
require 'benchmark'
require 'kenshin'

code = File.read('large_file.rb')

Benchmark.bm do |x|
  x.report("format:") { Kenshin.format(code) }
end
```

### Memory Profiling

```bash
# Install tool
gem install memory_profiler

# Create profile script
cat > profile_memory.rb <<'EOF'
require 'memory_profiler'
require 'kenshin'

code = File.read('large_file.rb')

report = MemoryProfiler.report do
  Kenshin.format(code)
end

report.pretty_print
EOF

# Run it
ruby profile_memory.rb
```

## Continuous Integration

Example GitHub Actions workflow (`.github/workflows/ci.yml`):

```yaml
name: CI

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        ruby: ['3.3', '3.4', '4.0']

    steps:
    - uses: actions/checkout@v4

    - uses: ruby/setup-ruby@v1
      with:
        ruby-version: ${{ matrix.ruby }}
        bundler-cache: true

    - uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
        override: true

    - name: Install dependencies
      run: bundle install

    - name: Build extension
      run: bundle exec rake compile

    - name: Run Ruby tests
      run: bundle exec rspec

    - name: Run Rust tests
      run: cd ext/kenshin && cargo test
```

## Additional Resources

- [Rust Documentation](https://doc.rust-lang.org/book/)
- [Magnus Documentation](https://docs.rs/magnus/)
- [RSpec Documentation](https://rspec.info/documentation/)
- [RubyGems Guides](https://guides.rubygems.org/)

## Getting Help

- GitHub Issues: https://github.com/yourusername/kenshin/issues
- Discussions: https://github.com/yourusername/kenshin/discussions
