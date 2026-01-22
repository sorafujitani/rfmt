---
title: "Development | fs0414/rfmt | DeepWiki"
source_url: "https://deepwiki.com/fs0414/rfmt/6-development"
fetched_at: "2026-01-13T10:53:43.101777+00:00"
---



Loading...

Index your code with Devin

[DeepWiki](https://deepwiki.com/)

[DeepWiki](https://deepwiki.com/)

[fs0414/rfmt](https://github.com/fs0414/rfmt "Open repository")

Index your code with

Devin

Edit WikiShare

Loading...

Last indexed: 3 January 2026 ([90c575](https://github.com/fs0414/rfmt/commits/90c575e6))

* [Overview](https://deepwiki.com/fs0414/rfmt/1-overview.html)
* [Architecture Overview](https://deepwiki.com/fs0414/rfmt/1.1-architecture-overview.html)
* [Key Concepts](https://deepwiki.com/fs0414/rfmt/1.2-key-concepts.html)
* [Getting Started](https://deepwiki.com/fs0414/rfmt/2-getting-started.html)
* [Installation](https://deepwiki.com/fs0414/rfmt/2.1-installation.html)
* [Configuration](https://deepwiki.com/fs0414/rfmt/2.2-configuration.html)
* [User Guides](https://deepwiki.com/fs0414/rfmt/3-user-guides.html)
* [CLI Usage](https://deepwiki.com/fs0414/rfmt/3.1-cli-usage.html)
* [Ruby API](https://deepwiki.com/fs0414/rfmt/3.2-ruby-api.html)
* [Editor Integration](https://deepwiki.com/fs0414/rfmt/3.3-editor-integration.html)
* [Core Systems](https://deepwiki.com/fs0414/rfmt/4-core-systems.html)
* [AST Representation](https://deepwiki.com/fs0414/rfmt/4.1-ast-representation.html)
* [Formatting Engine (Emitter)](https://deepwiki.com/fs0414/rfmt/4.2-formatting-engine-(emitter).html)
* [Parser Integration (PrismBridge)](https://deepwiki.com/fs0414/rfmt/4.3-parser-integration-(prismbridge).html)
* [Configuration System](https://deepwiki.com/fs0414/rfmt/4.4-configuration-system.html)
* [Caching System](https://deepwiki.com/fs0414/rfmt/4.5-caching-system.html)
* [Logging System](https://deepwiki.com/fs0414/rfmt/4.6-logging-system.html)
* [Formatting Behavior](https://deepwiki.com/fs0414/rfmt/5-formatting-behavior.html)
* [Control Flow Formatting](https://deepwiki.com/fs0414/rfmt/5.1-control-flow-formatting.html)
* [Exception Handling Formatting](https://deepwiki.com/fs0414/rfmt/5.2-exception-handling-formatting.html)
* [Lambda and Block Formatting](https://deepwiki.com/fs0414/rfmt/5.3-lambda-and-block-formatting.html)
* [Development](https://deepwiki.com/fs0414/rfmt/6-development.html)
* [Building from Source](https://deepwiki.com/fs0414/rfmt/6.1-building-from-source.html)
* [Testing](https://deepwiki.com/fs0414/rfmt/6.2-testing.html)
* [Release Process](https://deepwiki.com/fs0414/rfmt/6.3-release-process.html)
* [Reference](https://deepwiki.com/fs0414/rfmt/7-reference.html)
* [Node Types Reference](https://deepwiki.com/fs0414/rfmt/7.1-node-types-reference.html)
* [Configuration Options Reference](https://deepwiki.com/fs0414/rfmt/7.2-configuration-options-reference.html)
* [Error Codes Reference](https://deepwiki.com/fs0414/rfmt/7.3-error-codes-reference.html)
* [Performance and Benchmarks](https://deepwiki.com/fs0414/rfmt/7.4-performance-and-benchmarks.html)

Menu

# Development

Relevant source files

* [Cargo.lock](https://github.com/fs0414/rfmt/blob/90c575e6/Cargo.lock)
* [RELEASE.ja.md](https://github.com/fs0414/rfmt/blob/90c575e6/RELEASE.ja.md)
* [ext/rfmt/Cargo.toml](https://github.com/fs0414/rfmt/blob/90c575e6/ext/rfmt/Cargo.toml)
* [lib/rfmt/version.rb](https://github.com/fs0414/rfmt/blob/90c575e6/lib/rfmt/version.rb)

This document provides comprehensive guidance for developers contributing to rfmt or building it from source. It covers the development environment setup, build system architecture, testing procedures, and release workflows. For information about using rfmt as an end-user, see [CLI Usage](https://deepwiki.com/fs0414/rfmt/3.1-cli-usage.html). For details about rfmt's internal architecture and components, see [Core Systems](https://deepwiki.com/fs0414/rfmt/4-core-systems.html).

## Overview

rfmt's hybrid Ruby-Rust architecture requires a specialized development workflow. The Ruby layer ([lib/rfmt/](https://github.com/fs0414/rfmt/blob/90c575e6/lib/rfmt/)) provides the CLI, configuration, and AST serialization, while the Rust core ([ext/rfmt/src/](https://github.com/fs0414/rfmt/blob/90c575e6/ext/rfmt/src/)) implements the formatting engine. The build system bridges these two languages through `rb-sys`, which compiles Rust code into native extensions loadable by Ruby.

Development requires both Ruby and Rust toolchains, along with platform-specific build tools. The project uses automated quality gates (RuboCop, cargo clippy, RSpec, cargo test) enforced through Git hooks via Lefthook. The release process generates platform-specific native gems (Linux x86\_64, macOS ARM64/x86\_64, Windows x64) that can be distributed through RubyGems.org.

**Sources:** [ext/rfmt/Cargo.toml1-60](https://github.com/fs0414/rfmt/blob/90c575e6/ext/rfmt/Cargo.toml#L1-L60) [RELEASE.ja.md1-182](https://github.com/fs0414/rfmt/blob/90c575e6/RELEASE.ja.md#L1-L182)

## Development Environment

### Required Tools

| Tool | Minimum Version | Purpose |
| --- | --- | --- |
| Ruby | 3.0+ | Ruby runtime and gem management |
| Rust | 1.70+ | Compiling the Rust formatting core |
| Bundler | 2.0+ | Ruby dependency management |
| Cargo | Latest | Rust package management and build |
| RuboCop | Latest | Ruby code linting |
| RSpec | Latest | Ruby testing framework |

### Platform-Specific Requirements

**macOS:**

* Xcode Command Line Tools: `xcode-select --install`
* Homebrew (optional): For installing Ruby/Rust

**Linux:**

* GCC or Clang compiler
* Development headers: `build-essential` (Debian/Ubuntu) or `base-devel` (Arch)
* Ruby development headers: `ruby-dev` or `ruby-devel`

**Windows:**

* MSYS2 with MinGW-w64 toolchain
* Visual Studio Build Tools (for some dependencies)

### Initial Setup

The `rake compile` task invokes `rb-sys-build`, which compiles [ext/rfmt/src/](https://github.com/fs0414/rfmt/blob/90c575e6/ext/rfmt/src/) into a native shared library (`.so`, `.dylib`, or `.dll` depending on platform) that Ruby can load.

**Sources:** [ext/rfmt/Cargo.toml1-60](https://github.com/fs0414/rfmt/blob/90c575e6/ext/rfmt/Cargo.toml#L1-L60) [RELEASE.ja.md68-73](https://github.com/fs0414/rfmt/blob/90c575e6/RELEASE.ja.md#L68-L73)

## Build System Architecture

### Version Synchronization

rfmt requires version numbers to be synchronized across multiple files:

| File | Purpose | Format |
| --- | --- | --- |
| [lib/rfmt/version.rb4](https://github.com/fs0414/rfmt/blob/90c575e6/lib/rfmt/version.rb#L4-L4) | Ruby module constant | `VERSION = '1.2.3'` |
| [ext/rfmt/Cargo.toml3](https://github.com/fs0414/rfmt/blob/90c575e6/ext/rfmt/Cargo.toml#L3-L3) | Rust package version | `version = "1.2.3"` |
| [Cargo.lock](https://github.com/fs0414/rfmt/blob/90c575e6/Cargo.lock) | Dependency lock | Auto-generated by `cargo build` |

Any version change must update both `version.rb` and `Cargo.toml` manually, then run `cargo build` to regenerate `Cargo.lock`.

**Sources:** [lib/rfmt/version.rb1-6](https://github.com/fs0414/rfmt/blob/90c575e6/lib/rfmt/version.rb#L1-L6) [ext/rfmt/Cargo.toml1-10](https://github.com/fs0414/rfmt/blob/90c575e6/ext/rfmt/Cargo.toml#L1-L10)

### Build Targets

The build system supports multiple targets:

Each platform-specific gem task cross-compiles the Rust extension for the target platform and packages it with the Ruby code. The resulting gems include pre-compiled native extensions, eliminating the need for users to have Rust installed.

**Sources:** [RELEASE.ja.md84-109](https://github.com/fs0414/rfmt/blob/90c575e6/RELEASE.ja.md#L84-L109)

## Development Workflow

### Typical Development Commands

### Directory Structure

```
rfmt/
├── lib/rfmt/           # Ruby code
│   ├── cli.rb          # CLI implementation
│   ├── config.rb       # Configuration system
│   ├── prism_bridge.rb # AST serialization
│   └── version.rb      # Version constant
├── ext/rfmt/           # Rust extension
│   ├── src/
│   │   ├── lib.rs      # FFI entry points
│   │   ├── ast/        # AST definitions
│   │   ├── emitter/    # Formatting engine
│   │   ├── config/     # Configuration loader
│   │   └── logging/    # Logger implementation
│   ├── Cargo.toml      # Rust manifest
│   └── Cargo.lock      # Dependency lock
├── spec/               # RSpec tests
└── exe/rfmt            # CLI entry point
```

**Sources:** [lib/rfmt/version.rb1-6](https://github.com/fs0414/rfmt/blob/90c575e6/lib/rfmt/version.rb#L1-L6) [ext/rfmt/Cargo.toml1-60](https://github.com/fs0414/rfmt/blob/90c575e6/ext/rfmt/Cargo.toml#L1-L60)

## Quality Gates

### Automated Checks

rfmt enforces code quality through multiple automated gates:

### RuboCop Configuration

RuboCop enforces Ruby style guidelines. Configuration is typically in `.rubocop.yml`:

* Maximum line length: 120 characters (aligned with `line_length` config)
* Style: Ruby 3.0+ syntax preferred
* Frozen string literals: Required
* Documentation: Required for public APIs

Run `bundle exec rubocop -a` to auto-fix most style violations.

### Cargo Clippy Rules

Clippy enforces Rust best practices defined in [ext/rfmt/Cargo.toml](https://github.com/fs0414/rfmt/blob/90c575e6/ext/rfmt/Cargo.toml):

* Pedantic warnings enabled for stricter checks
* Nursery lints for experimental checks
* Unsafe code minimized (only in FFI boundary via magnus)
* Error handling: All errors must use `thiserror` or `anyhow`

Run `cd ext/rfmt && cargo clippy -- -D warnings` to treat warnings as errors.

### Test Coverage

**Ruby Tests (RSpec):**

* Location: [spec/](https://github.com/fs0414/rfmt/blob/90c575e6/spec/)
* Run: `bundle exec rspec`
* Coverage: Integration tests for CLI, configuration, AST serialization
* Mock external dependencies (Prism parser) where appropriate

**Rust Tests (cargo test):**

* Location: [ext/rfmt/src/](https://github.com/fs0414/rfmt/blob/90c575e6/ext/rfmt/src/) (inline with source)
* Run: `cd ext/rfmt && cargo test`
* Coverage: Unit tests for AST structures, formatting rules, configuration parsing
* Integration tests in [ext/rfmt/tests/](https://github.com/fs0414/rfmt/blob/90c575e6/ext/rfmt/tests/) (if present)

**Sources:** [ext/rfmt/Cargo.toml50-56](https://github.com/fs0414/rfmt/blob/90c575e6/ext/rfmt/Cargo.toml#L50-L56)

### Lefthook Integration

Lefthook manages Git hooks defined in `lefthook.yml`:

**Pre-commit Hook:**

* Runs `cargo fmt --check` to ensure Rust code is formatted
* Runs `rubocop` on staged Ruby files
* Prevents commits with formatting issues

**Pre-push Hook:**

* Runs full test suite (RSpec + cargo test)
* Runs linters (RuboCop + clippy)
* Blocks push if any checks fail

Install hooks: `lefthook install`

**Sources:** [RELEASE.ja.md1-182](https://github.com/fs0414/rfmt/blob/90c575e6/RELEASE.ja.md#L1-L182)

## Dependency Management

### Ruby Dependencies

Dependencies are managed through [Gemfile](https://github.com/fs0414/rfmt/blob/90c575e6/Gemfile) and [rfmt.gemspec](https://github.com/fs0414/rfmt/blob/90c575e6/rfmt.gemspec):

**Development Dependencies:**

* `rspec`: Testing framework
* `rubocop`: Linting and style checking
* `lefthook`: Git hook management
* `rake`: Build automation
* `rb_sys`: Rust extension builder

**Runtime Dependencies:**

* `prism`: Ruby parser (generates AST)
* Minimal to keep gem lightweight

Update dependencies: `bundle update`

### Rust Dependencies

Dependencies are declared in [ext/rfmt/Cargo.toml13-45](https://github.com/fs0414/rfmt/blob/90c575e6/ext/rfmt/Cargo.toml#L13-L45):

| Dependency | Version | Purpose |
| --- | --- | --- |
| `magnus` | 0.6.2 | Ruby-Rust FFI bridge |
| `rb-sys` | 0.9.117 | Ruby C API bindings |
| `serde` | 1.0 | Serialization framework |
| `serde_json` | 1.0 | JSON serialization (AST transfer) |
| `serde_yaml` | 0.9 | YAML config parsing |
| `rayon` | 1.8 | Parallel processing |
| `log` | 0.4 | Logging facade |
| `env_logger` | 0.11 | Logger implementation |
| `anyhow` | 1.0 | Error handling |
| `thiserror` | 1.0 | Error type derivation |
| `lru` | 0.12 | LRU cache |
| `globset` | 0.4 | Glob pattern matching |
| `dirs` | 5.0 | Config file discovery |

**Development Dependencies:**

* `proptest`: Property-based testing
* `insta`: Snapshot testing
* `criterion`: Benchmarking
* `tempfile`: Temporary file management

Update dependencies: `cd ext/rfmt && cargo update`

**Sources:** [ext/rfmt/Cargo.toml13-56](https://github.com/fs0414/rfmt/blob/90c575e6/ext/rfmt/Cargo.toml#L13-L56)

## Release Process

### Version Update Workflow

### Automated Release (Recommended)

GitHub Actions automatically builds and publishes gems when a version tag is pushed:

1. **Update versions** in [lib/rfmt/version.rb](https://github.com/fs0414/rfmt/blob/90c575e6/lib/rfmt/version.rb) and [ext/rfmt/Cargo.toml](https://github.com/fs0414/rfmt/blob/90c575e6/ext/rfmt/Cargo.toml)
2. **Update CHANGELOG.md** with release notes
3. **Commit changes**: `git commit -m "Bump version to X.Y.Z"`
4. **Create and push tag**: `git tag vX.Y.Z && git push origin vX.Y.Z`
5. **Monitor GitHub Actions** at `https://github.com/fs0414/rfmt/actions`

The CI workflow compiles native extensions for all supported platforms in parallel and publishes them to RubyGems.org (requires `RUBYGEMS_API_KEY` secret configured).

**Sources:** [RELEASE.ja.md5-63](https://github.com/fs0414/rfmt/blob/90c575e6/RELEASE.ja.md#L5-L63)

### Manual Release

For manual release or when CI is unavailable:

1. **Update versions** as described above
2. **Build platform-specific gems**:
3. **Publish each gem**:
4. **Create GitHub Release** with tag `vX.Y.Z` and CHANGELOG content

**Sources:** [RELEASE.ja.md64-139](https://github.com/fs0414/rfmt/blob/90c575e6/RELEASE.ja.md#L64-L139)

### Release Checklist

| Step | File/Command | Verification |
| --- | --- | --- |
| Update Ruby version | [lib/rfmt/version.rb4](https://github.com/fs0414/rfmt/blob/90c575e6/lib/rfmt/version.rb#L4-L4) | `grep VERSION lib/rfmt/version.rb` |
| Update Rust version | [ext/rfmt/Cargo.toml3](https://github.com/fs0414/rfmt/blob/90c575e6/ext/rfmt/Cargo.toml#L3-L3) | `grep '^version' ext/rfmt/Cargo.toml` |
| Regenerate lock file | `cargo build` | `git status` shows `Cargo.lock` changed |
| Update changelog | `CHANGELOG.md` | Add section `## [X.Y.Z] - YYYY-MM-DD` |
| Commit changes | `git commit -am "Bump version"` | Clean working directory |
| Create tag | `git tag vX.Y.Z` | `git tag -l` shows new tag |
| Push tag | `git push origin vX.Y.Z` | GitHub shows new tag |
| Verify CI | GitHub Actions | All builds pass |
| Verify publication | RubyGems.org | New version appears |
| Test installation | `gem install rfmt` | Correct version installs |

**Sources:** [lib/rfmt/version.rb1-6](https://github.com/fs0414/rfmt/blob/90c575e6/lib/rfmt/version.rb#L1-L6) [ext/rfmt/Cargo.toml1-10](https://github.com/fs0414/rfmt/blob/90c575e6/ext/rfmt/Cargo.toml#L1-L10) [RELEASE.ja.md140-156](https://github.com/fs0414/rfmt/blob/90c575e6/RELEASE.ja.md#L140-L156)

## Troubleshooting

### Build Failures

**Problem:** `rake compile` fails with "rb-sys not found"

**Solution:**

**Problem:** Rust compilation errors after dependency update

**Solution:**

**Problem:** Platform-specific gem build fails

**Solution:** Ensure target platform toolchain is installed:

* Linux: `sudo apt-get install build-essential ruby-dev`
* macOS: `xcode-select --install`
* Windows: Install MSYS2 and MinGW-w64

### Test Failures

**Problem:** RSpec tests fail with "cannot load such file -- rfmt"

**Solution:** Rebuild the extension: `rake compile`

**Problem:** Cargo tests fail with linking errors

**Solution:** Clean and rebuild:

### Release Issues

**Problem:** GitHub Actions fails to publish gems

**Solution:** Verify `RUBYGEMS_API_KEY` secret:

1. Go to Settings → Secrets → Actions
2. Ensure `RUBYGEMS_API_KEY` is set with valid API key from rubygems.org
3. Key should have `push_rubygem` scope

**Problem:** Manual gem push fails with authentication error

**Solution:** Sign in again: `gem signin` and re-enter credentials

**Sources:** [RELEASE.ja.md157-176](https://github.com/fs0414/rfmt/blob/90c575e6/RELEASE.ja.md#L157-L176)

Dismiss

Refresh this wiki

Enter email to refresh

### On this page

* [Development](https://deepwiki.com/fs0414/rfmt/6-development.html#development)
* [Overview](https://deepwiki.com/fs0414/rfmt/6-development.html#overview)
* [Development Environment](https://deepwiki.com/fs0414/rfmt/6-development.html#development-environment)
* [Required Tools](https://deepwiki.com/fs0414/rfmt/6-development.html#required-tools)
* [Platform-Specific Requirements](https://deepwiki.com/fs0414/rfmt/6-development.html#platform-specific-requirements)
* [Initial Setup](https://deepwiki.com/fs0414/rfmt/6-development.html#initial-setup)
* [Build System Architecture](https://deepwiki.com/fs0414/rfmt/6-development.html#build-system-architecture)
* [Version Synchronization](https://deepwiki.com/fs0414/rfmt/6-development.html#version-synchronization)
* [Build Targets](https://deepwiki.com/fs0414/rfmt/6-development.html#build-targets)
* [Development Workflow](https://deepwiki.com/fs0414/rfmt/6-development.html#development-workflow)
* [Typical Development Commands](https://deepwiki.com/fs0414/rfmt/6-development.html#typical-development-commands)
* [Directory Structure](https://deepwiki.com/fs0414/rfmt/6-development.html#directory-structure)
* [Quality Gates](https://deepwiki.com/fs0414/rfmt/6-development.html#quality-gates)
* [Automated Checks](https://deepwiki.com/fs0414/rfmt/6-development.html#automated-checks)
* [RuboCop Configuration](https://deepwiki.com/fs0414/rfmt/6-development.html#rubocop-configuration)
* [Cargo Clippy Rules](https://deepwiki.com/fs0414/rfmt/6-development.html#cargo-clippy-rules)
* [Test Coverage](https://deepwiki.com/fs0414/rfmt/6-development.html#test-coverage)
* [Lefthook Integration](https://deepwiki.com/fs0414/rfmt/6-development.html#lefthook-integration)
* [Dependency Management](https://deepwiki.com/fs0414/rfmt/6-development.html#dependency-management)
* [Ruby Dependencies](https://deepwiki.com/fs0414/rfmt/6-development.html#ruby-dependencies)
* [Rust Dependencies](https://deepwiki.com/fs0414/rfmt/6-development.html#rust-dependencies)
* [Release Process](https://deepwiki.com/fs0414/rfmt/6-development.html#release-process)
* [Version Update Workflow](https://deepwiki.com/fs0414/rfmt/6-development.html#version-update-workflow)
* [Automated Release (Recommended)](https://deepwiki.com/fs0414/rfmt/6-development.html#automated-release-recommended)
* [Manual Release](https://deepwiki.com/fs0414/rfmt/6-development.html#manual-release)
* [Release Checklist](https://deepwiki.com/fs0414/rfmt/6-development.html#release-checklist)
* [Troubleshooting](https://deepwiki.com/fs0414/rfmt/6-development.html#troubleshooting)
* [Build Failures](https://deepwiki.com/fs0414/rfmt/6-development.html#build-failures)
* [Test Failures](https://deepwiki.com/fs0414/rfmt/6-development.html#test-failures)
* [Release Issues](https://deepwiki.com/fs0414/rfmt/6-development.html#release-issues)
