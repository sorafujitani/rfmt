# Development Guide

This guide covers testing, building, and releasing rfmt.

## Table of Contents

- [Prerequisites](#prerequisites)
  - [Nix Development Environment](#nix-development-environment)
- [Building](#building)
- [Testing](#testing)
- [Development Workflow](#development-workflow)
- [Release Process](#release-process)
- [Troubleshooting](#troubleshooting)

## Prerequisites

### Required Tools

- **Ruby**: 3.0 or later
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

### Nix Development Environment

**Nix Flake (Recommended)** - Complete isolated development environment:

```bash
# Prerequisites: Nix with flakes enabled
# Install Nix (if not already installed):
# curl --proto '=https' --tlsv1.2 -sSf https://install.determinate.systems/nix | sh

# Clone repository
git clone https://github.com/yourusername/rfmt.git
cd rfmt

# Enter development environment
nix develop --impure
```

**With direnv (Auto-loading)**:

```bash
# Install direnv (if not already installed)
nix profile install nixpkgs#direnv

# Setup direnv integration
echo "use flake" > .envrc
direnv allow

# Environment will automatically load when entering the directory
cd rfmt
# 🚀 rfmt development environment loaded!
```

**What Nix Provides:**
- ✅ Ruby 3.4.5 (matching mise.toml)
- ✅ Rust 1.70+ with cargo, clippy, rustfmt
- ✅ System dependencies (pkg-config, openssl, etc.)
- ✅ Build tools (gcc, make, bundler)
- ✅ Development utilities
- ✅ Isolated environment (no conflicts with system packages)

**Quick Commands:**
```bash
# After entering nix develop or direnv environment:
bundle install              # Install Ruby dependencies
bundle exec rake compile    # Compile Rust extension
bundle exec rspec           # Run Ruby tests
cargo test --manifest-path ext/rfmt/Cargo.toml  # Run Rust tests

# Development aliases (available in direnv)
rspec                       # bundle exec rspec
rfmt-test                   # Run all tests
rfmt-lint                   # Run all linters
rfmt-format                 # Format all code
```

**Nix Apps (Development Scripts):**
```bash
# Setup script (installs direnv if needed)
nix run .#setup

# Test runner (full test suite)
nix run .#test
```

### Manual Setup

```bash
# Clone repository
git clone https://github.com/yourusername/rfmt.git
cd rfmt

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
cd ext/rfmt
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
- Version command (`rfmt version`)
- Format command with various options (`--write`, `--no-write`, `--check`, `--diff`, `--verbose`)
- Check mode with proper exit codes (0 for formatted, 1 for needs formatting)
- Diff display in 3 formats (unified, color, side_by_side)
- Multiple file processing
- Error handling (syntax errors, missing files)
- Init command (`.rfmt.yml` creation)
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
bundle exec rspec spec/configuration_spec.rb -e "discovers .rfmt.yml"
```

**Configuration Test Coverage:**
- Auto-discovery of config files (`.rfmt.yml`, `.rfmt.yaml`, `rfmt.yml`, `rfmt.yaml`)
- Default configuration loading
- Custom configuration file loading
- Configuration merging (deep merge for nested hashes)
- Validation (line_length > 0, indent_width > 0)
- File pattern matching (include/exclude)
- Formatting options retrieval

**Example Test Cases:**
```ruby
# Test config file discovery
it 'discovers .rfmt.yml' do
  File.write('.rfmt.yml', "version: '1.0'")
  config = described_class.discover
  expect(config).to be_a(described_class)
end

# Test configuration validation
it 'validates positive line_length' do
  expect do
    described_class.new('formatting' => { 'line_length' => -1 })
  end.to raise_error(Rfmt::Configuration::ConfigError, 'line_length must be positive')
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
cd ext/rfmt

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

### Coverage

```bash
# Install coverage tool (once)
cargo install cargo-tarpaulin

# Generate coverage report
cd ext/rfmt
cargo tarpaulin --out Html --output-dir ../../coverage
```

## Development Workflow

### 1. Make Changes

Edit files in:
- `lib/` - Ruby code
- `ext/rfmt/src/` - Rust code
- `spec/` - Tests

### 2. Build & Test

```bash
# After changing Rust code
bundle exec rake compile

# Run tests
bundle exec rake spec

# Run Rust tests
cd ext/rfmt && cargo test
```

### 3. Verify

```bash
# Manual testing with IRB
bundle exec irb -I lib -r rfmt

# In IRB:
input = "class Foo\nend"
puts Rfmt.format(input)
```

### 4. Format & Lint

```bash
# Format Rust code
cd ext/rfmt
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
cd ext/rfmt && cargo test
```

## Release Process

### Pre-Release Checklist

- [ ] All tests passing
- [ ] Version updated in `lib/rfmt/version.rb`
- [ ] Version updated in `ext/rfmt/Cargo.toml`
- [ ] CHANGELOG.md updated
- [ ] Documentation updated
- [ ] No uncommitted changes

### Version Update

1. **Update Ruby version** (`lib/rfmt/version.rb`):

```ruby
module Rfmt
  VERSION = "0.2.0"  # Update this
end
```

2. **Update Rust version** (`ext/rfmt/Cargo.toml`):

```toml
[package]
name = "rfmt"
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
gem build rfmt.gemspec

# This creates: rfmt-0.2.0.gem
```

### Test Gem Locally

```bash
# Install locally
gem install rfmt-0.2.0.gem

# Test it
irb
> require 'rfmt'
> Rfmt.format("class Foo\nend")
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
gem push rfmt-0.2.0.gem

# Verify at https://rubygems.org/gems/rfmt
```

### Post-Release

1. **Create Git Tag**:

```bash
git tag -a v0.2.0 -m "Release version 0.2.0"
git push origin v0.2.0
```

2. **Create GitHub Release**:

- Go to https://github.com/yourusername/rfmt/releases/new
- Select tag `v0.2.0`
- Set release title: `v0.2.0`
- Copy CHANGELOG entry to description
- Attach `rfmt-0.2.0.gem` file
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
cd ext/rfmt && cargo update

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

#### Problem: "Cannot load such file -- rfmt/rfmt"

```bash
# Extension not built or not in correct location
bundle exec rake compile

# Check if extension exists
ls -la lib/rfmt/rfmt.bundle  # macOS
ls -la lib/rfmt/rfmt.so      # Linux
```

### Runtime Issues

#### Problem: "Prism integration error"

```bash
# Check Prism gem version
bundle list | grep prism

# Should be ~> 1.6.0
# Update if needed
bundle update prism
```

#### Problem: Segmentation fault

This usually indicates a bug in Rust code. To debug:

```bash
# Build debug version
cd ext/rfmt
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
file lib/rfmt/rfmt.bundle
# Should say "not stripped" for debug, "stripped" for release
```

### Nix Issues

#### Problem: "nix: command not found"

```bash
# Check if Nix is installed
ls -la /nix/var/nix/profiles/default/bin/nix

# If Nix is installed but not in PATH, add it temporarily:
export PATH="/nix/var/nix/profiles/default/bin:$PATH"

# To make it permanent, add to your shell profile:
echo 'export PATH="/nix/var/nix/profiles/default/bin:$PATH"' >> ~/.bashrc
# or for zsh:
echo 'export PATH="/nix/var/nix/profiles/default/bin:$PATH"' >> ~/.zshrc

# Source the updated profile
source ~/.bashrc  # or ~/.zshrc
```

#### Problem: "flake.nix is not tracked by Git"

```bash
# Nix flakes require files to be tracked by Git
git add flake.nix .envrc
git commit -m "Add Nix flake development environment"
```

#### Problem: First `nix develop` takes too long

```bash
# First run downloads and builds many packages, this is normal
# Subsequent runs will use cached builds

# To see progress:
nix develop --impure --show-trace

# For faster subsequent builds, consider using:
nix develop --impure --offline  # Use only cached packages
```

#### Problem: direnv not auto-loading

```bash
# Make sure direnv is installed
nix profile install nixpkgs#direnv

# Hook direnv into your shell (add to ~/.bashrc or ~/.zshrc):
eval "$(direnv hook bash)"  # for bash
eval "$(direnv hook zsh)"   # for zsh

# Allow direnv for the project
cd rfmt
direnv allow

# Force reload
direnv reload
```

#### Problem: Ruby gems fail to install in Nix environment

```bash
# Clear gem cache and reinstall
rm -rf .nix-gem-home
nix develop --impure --command bundle install

# If native extension compilation fails:
nix develop --impure --command bundle config build.eventmachine --with-cppflags=-I/nix/store/.../include
```

#### Problem: Rust compilation fails in Nix

```bash
# Check Rust toolchain
nix develop --impure --command rustc --version
nix develop --impure --command cargo --version

# Clear Rust cache and rebuild
rm -rf .nix-cargo-home
nix develop --impure --command cargo clean --manifest-path ext/rfmt/Cargo.toml
nix develop --impure --command bundle exec rake compile
```

#### Problem: "darwin.apple_sdk" errors on macOS

This is typically resolved in our flake, but if you encounter issues:

```bash
# Check if you're using an outdated nixpkgs
nix flake update

# Force rebuild with updated dependencies
nix develop --impure --rebuild
```

## Development Tips

### Fast Iteration

```bash
# Terminal 1: Watch for file changes
while true; do
  inotifywait -e modify ext/rfmt/src/*.rs
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
require 'rfmt'

code = File.read('large_file.rb')

Benchmark.bm do |x|
  x.report("format:") { Rfmt.format(code) }
end
```

### Memory Profiling

```bash
# Install tool
gem install memory_profiler

# Create profile script
cat > profile_memory.rb <<'EOF'
require 'memory_profiler'
require 'rfmt'

code = File.read('large_file.rb')

report = MemoryProfiler.report do
  Rfmt.format(code)
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
        ruby: ['3.0', '3.1', '3.2', '3.3']

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
      run: cd ext/rfmt && cargo test
```

### Nix CI Workflow

Example GitHub Actions workflow using Nix (`.github/workflows/nix-ci.yml`):

```yaml
name: Nix CI

on: [push, pull_request]

jobs:
  nix-test:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v4
    
    - name: Install Nix
      uses: DeterminateSystems/nix-installer-action@main
      with:
        enable-flakes: true
    
    - name: Setup Nix Cache
      uses: DeterminateSystems/magic-nix-cache-action@main
    
    - name: Check flake
      run: nix flake check --no-build
    
    - name: Run tests in Nix environment
      run: nix run .#test
    
    - name: Check formatting
      run: |
        nix develop --impure --command bundle exec rubocop --check
        nix develop --impure --command cargo fmt --manifest-path ext/rfmt/Cargo.toml -- --check
```

## Additional Resources

- [Rust Documentation](https://doc.rust-lang.org/book/)
- [Magnus Documentation](https://docs.rs/magnus/)
- [RSpec Documentation](https://rspec.info/documentation/)
- [RubyGems Guides](https://guides.rubygems.org/)

## Getting Help

- GitHub Issues: https://github.com/yourusername/rfmt/issues
- Discussions: https://github.com/yourusername/rfmt/discussions
