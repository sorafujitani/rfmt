---
title: "Getting Started | fs0414/rfmt | DeepWiki"
source_url: "https://deepwiki.com/fs0414/rfmt/2-getting-started"
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

# Getting Started

Relevant source files

* [CHANGELOG.md](https://github.com/fs0414/rfmt/blob/90c575e6/CHANGELOG.md)
* [Gemfile.lock](https://github.com/fs0414/rfmt/blob/90c575e6/Gemfile.lock)
* [README.md](https://github.com/fs0414/rfmt/blob/90c575e6/README.md)
* [docs/user\_guide.ja.md](https://github.com/fs0414/rfmt/blob/90c575e6/docs/user_guide.ja.md)
* [docs/user\_guide.md](https://github.com/fs0414/rfmt/blob/90c575e6/docs/user_guide.md)

This page guides you through installing rfmt, creating your first configuration, and formatting your first Ruby file. For detailed information about installation methods and platform-specific considerations, see [Installation](https://deepwiki.com/fs0414/rfmt/2.1-installation.html). For comprehensive configuration options and precedence rules, see [Configuration](https://deepwiki.com/fs0414/rfmt/2.2-configuration.html).

## Overview

The getting started process involves three steps:

1. **Installation** - Install the `rfmt` gem via RubyGems
2. **Configuration** - Create a `.rfmt.yml` file with `rfmt init`
3. **First Format** - Run `rfmt` on a Ruby file

This typically takes less than 5 minutes to complete.

## Prerequisites

Verify your system meets the minimum requirements:

| Requirement | Minimum Version | Purpose |
| --- | --- | --- |
| Ruby | 3.0 | Runtime environment |
| RubyGems | (bundled with Ruby) | Package installation |
| Rust | 1.70 | Building from source only |

Check your Ruby version:

**Sources:** [README.md79-82](https://github.com/fs0414/rfmt/blob/90c575e6/README.md#L79-L82)

---

## Installation Flow

The following diagram shows how rfmt is installed and initialized on your system:

**Sources:** [README.md77-109](https://github.com/fs0414/rfmt/blob/90c575e6/README.md#L77-L109) [Gemfile.lock1-6](https://github.com/fs0414/rfmt/blob/90c575e6/Gemfile.lock#L1-L6) [Gemfile.lock99-101](https://github.com/fs0414/rfmt/blob/90c575e6/Gemfile.lock#L99-L101)

---

## Quick Installation

### Method 1: RubyGems (Recommended)

Install rfmt globally:

This downloads a pre-compiled native gem for your platform (Linux, macOS, or Windows) from RubyGems.org. The native gem includes the Rust binary [ext/rfmt/rfmt.so](https://github.com/fs0414/rfmt/blob/90c575e6/ext/rfmt/rfmt.so) compiled for your architecture.

Verify installation:

### Method 2: Bundler (Project-Specific)

Add to your `Gemfile`:

Install:

The gem will be installed in your project's bundle. Access via:

### Method 3: From Source (Development)

For contributing or building on unsupported platforms:

The `rake compile` task invokes [rb-sys-build](https://github.com/fs0414/rfmt/blob/90c575e6/rb-sys-build) to compile the Rust extension in [ext/rfmt/src/](https://github.com/fs0414/rfmt/blob/90c575e6/ext/rfmt/src/) using Cargo.

**Sources:** [README.md84-109](https://github.com/fs0414/rfmt/blob/90c575e6/README.md#L84-L109) [Gemfile.lock4-5](https://github.com/fs0414/rfmt/blob/90c575e6/Gemfile.lock#L4-L5) [Gemfile.lock38-42](https://github.com/fs0414/rfmt/blob/90c575e6/Gemfile.lock#L38-L42)

---

## Configuration Discovery

The following diagram shows how `Rfmt::Config` discovers configuration files:

**Sources:** [README.md220-228](https://github.com/fs0414/rfmt/blob/90c575e6/README.md#L220-L228) [docs/user\_guide.md120-127](https://github.com/fs0414/rfmt/blob/90c575e6/docs/user_guide.md#L120-L127)

---

## Creating Your First Configuration

Generate a default configuration file:

This creates `.rfmt.yml` in the current directory with default values. The file is generated by `Rfmt::Config.init` and contains the following structure:

### Configuration Options Summary

| Option | Type | Default | Description |
| --- | --- | --- | --- |
| `formatting.line_length` | Integer | 100 | Maximum line width before wrapping |
| `formatting.indent_width` | Integer | 2 | Number of spaces/tabs per indent level |
| `formatting.indent_style` | String | "spaces" | Use "spaces" or "tabs" for indentation |
| `formatting.quote_style` | String | "double" | String quote style: "double", "single", or "consistent" |
| `include` | Array | See above | Glob patterns for files to format |
| `exclude` | Array | See above | Glob patterns for files to skip |

**Options:**

**Sources:** [README.md113-153](https://github.com/fs0414/rfmt/blob/90c575e6/README.md#L113-L153) [docs/user\_guide.md86-118](https://github.com/fs0414/rfmt/blob/90c575e6/docs/user_guide.md#L86-L118)

---

## First Format Operation

The following diagram shows the execution flow when you run `rfmt` for the first time:

**Sources:** [README.md156-180](https://github.com/fs0414/rfmt/blob/90c575e6/README.md#L156-L180) [README.md189-216](https://github.com/fs0414/rfmt/blob/90c575e6/README.md#L189-L216)

---

### Format a File

Format a single Ruby file in-place:

The CLI (`exe/rfmt`) invokes `Rfmt::CLI.run` which:

1. Loads configuration via `Rfmt::Config.load`
2. Parses the file with Prism
3. Serializes to JSON via `Rfmt::PrismBridge.parse`
4. Calls the Rust emitter via Magnus FFI
5. Writes the formatted output back to the file

**Before:**

**After:**

### Preview Changes (Dry Run)

Show differences without modifying the file:

This passes the `--diff` flag to `Rfmt::CLI`, which displays a unified diff using the `diffy` gem but does not write changes.

### Check Formatting (CI/CD)

Verify formatting without modifications:

Exit codes:

* `0`: File is already formatted
* `2`: File needs formatting
* `1`: Error occurred (parse error, file not found, etc.)

The `check` command is an alias for the `--check` flag in `Rfmt::CLI`.

**Sources:** [README.md156-186](https://github.com/fs0414/rfmt/blob/90c575e6/README.md#L156-L186) [docs/user\_guide.md43-83](https://github.com/fs0414/rfmt/blob/90c575e6/docs/user_guide.md#L43-L83)

---

## Ruby API Usage

For programmatic use in Ruby scripts, tests, or Rake tasks:

The `Rfmt.format` method:

1. Accepts a Ruby source string
2. Parses it via `Prism.parse`
3. Converts to JSON via `PrismBridge.parse`
4. Calls Rust emitter via Magnus FFI
5. Returns the formatted string

For AST inspection:

**Sources:** [README.md189-216](https://github.com/fs0414/rfmt/blob/90c575e6/README.md#L189-L216) [docs/user\_guide.md231-287](https://github.com/fs0414/rfmt/blob/90c575e6/docs/user_guide.md#L231-L287)

---

## Verification Steps

After installation and configuration, verify your setup:

### 1. Check Version

### 2. Verify Configuration

### 3. Test Format

Create a test file:

Format it:

Verify output:

### 4. Enable Debug Mode

If you encounter issues, enable verbose output:

Debug output shows:

* Configuration file discovery
* File processing details
* Rust emitter operations
* Cache operations

**Sources:** [README.md180-187](https://github.com/fs0414/rfmt/blob/90c575e6/README.md#L180-L187) [docs/user\_guide.md429-456](https://github.com/fs0414/rfmt/blob/90c575e6/docs/user_guide.md#L429-L456)

---

## Common First-Time Issues

| Issue | Cause | Solution |
| --- | --- | --- |
| `command not found: rfmt` | Gem not in PATH | Use `bundle exec rfmt` or add gem bin to PATH |
| `LoadError: cannot load such file -- rfmt` | Native extension not compiled | Run `bundle exec rake compile` |
| `Rfmt::ParseError: E001` | Ruby syntax error in file | Fix syntax before formatting |
| Config not found | `.rfmt.yml` not created | Run `rfmt init` first |
| No files formatted | Files excluded by config | Check `exclude` patterns in `.rfmt.yml` |

**Sources:** [docs/user\_guide.md389-456](https://github.com/fs0414/rfmt/blob/90c575e6/docs/user_guide.md#L389-L456)

---

## Next Steps

Now that rfmt is installed and configured:

* **[Installation](https://deepwiki.com/fs0414/rfmt/2.1-installation.html)** - Learn about platform-specific installation, native gems, and building from source
* **[Configuration](https://deepwiki.com/fs0414/rfmt/2.2-configuration.html)** - Explore advanced configuration options, precedence rules, and file filtering
* **[CLI Usage](https://deepwiki.com/fs0414/rfmt/3.1-cli-usage.html)** - Master all CLI commands, flags, and workflows
* **[Ruby API](https://deepwiki.com/fs0414/rfmt/3.2-ruby-api.html)** - Integrate rfmt programmatically in Ruby applications
* **[Editor Integration](https://deepwiki.com/fs0414/rfmt/3.3-editor-integration.html)** - Set up format-on-save in Neovim, VS Code, and other editors

**Sources:** [README.md1-398](https://github.com/fs0414/rfmt/blob/90c575e6/README.md#L1-L398)

Dismiss

Refresh this wiki

Enter email to refresh

### On this page

* [Getting Started](https://deepwiki.com/fs0414/rfmt/2-getting-started.html#getting-started)
* [Overview](https://deepwiki.com/fs0414/rfmt/2-getting-started.html#overview)
* [Prerequisites](https://deepwiki.com/fs0414/rfmt/2-getting-started.html#prerequisites)
* [Installation Flow](https://deepwiki.com/fs0414/rfmt/2-getting-started.html#installation-flow)
* [Quick Installation](https://deepwiki.com/fs0414/rfmt/2-getting-started.html#quick-installation)
* [Method 1: RubyGems (Recommended)](https://deepwiki.com/fs0414/rfmt/2-getting-started.html#method-1-rubygems-recommended)
* [Method 2: Bundler (Project-Specific)](https://deepwiki.com/fs0414/rfmt/2-getting-started.html#method-2-bundler-project-specific)
* [Method 3: From Source (Development)](https://deepwiki.com/fs0414/rfmt/2-getting-started.html#method-3-from-source-development)
* [Configuration Discovery](https://deepwiki.com/fs0414/rfmt/2-getting-started.html#configuration-discovery)
* [Creating Your First Configuration](https://deepwiki.com/fs0414/rfmt/2-getting-started.html#creating-your-first-configuration)
* [Configuration Options Summary](https://deepwiki.com/fs0414/rfmt/2-getting-started.html#configuration-options-summary)
* [First Format Operation](https://deepwiki.com/fs0414/rfmt/2-getting-started.html#first-format-operation)
* [Format a File](https://deepwiki.com/fs0414/rfmt/2-getting-started.html#format-a-file)
* [Preview Changes (Dry Run)](https://deepwiki.com/fs0414/rfmt/2-getting-started.html#preview-changes-dry-run)
* [Check Formatting (CI/CD)](https://deepwiki.com/fs0414/rfmt/2-getting-started.html#check-formatting-cicd)
* [Ruby API Usage](https://deepwiki.com/fs0414/rfmt/2-getting-started.html#ruby-api-usage)
* [Verification Steps](https://deepwiki.com/fs0414/rfmt/2-getting-started.html#verification-steps)
* [1. Check Version](https://deepwiki.com/fs0414/rfmt/2-getting-started.html#1-check-version)
* [2. Verify Configuration](https://deepwiki.com/fs0414/rfmt/2-getting-started.html#2-verify-configuration)
* [3. Test Format](https://deepwiki.com/fs0414/rfmt/2-getting-started.html#3-test-format)
* [4. Enable Debug Mode](https://deepwiki.com/fs0414/rfmt/2-getting-started.html#4-enable-debug-mode)
* [Common First-Time Issues](https://deepwiki.com/fs0414/rfmt/2-getting-started.html#common-first-time-issues)
* [Next Steps](https://deepwiki.com/fs0414/rfmt/2-getting-started.html#next-steps)
