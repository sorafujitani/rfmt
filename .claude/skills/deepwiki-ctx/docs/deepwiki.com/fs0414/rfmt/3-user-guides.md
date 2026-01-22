---
title: "User Guides | fs0414/rfmt | DeepWiki"
source_url: "https://deepwiki.com/fs0414/rfmt/3-user-guides"
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

# User Guides

Relevant source files

* [README.md](https://github.com/fs0414/rfmt/blob/90c575e6/README.md)
* [docs/user\_guide.ja.md](https://github.com/fs0414/rfmt/blob/90c575e6/docs/user_guide.ja.md)
* [docs/user\_guide.md](https://github.com/fs0414/rfmt/blob/90c575e6/docs/user_guide.md)

This page provides comprehensive guidance on using rfmt in different contexts and workflows. It covers the three primary interaction patterns: command-line usage, programmatic Ruby API integration, and editor integration. For installation instructions and initial setup, see [Installation](https://deepwiki.com/fs0414/rfmt/2.1-installation.html). For detailed configuration options, see [Configuration](https://deepwiki.com/fs0414/rfmt/2.2-configuration.html).

---

## Overview of Usage Patterns

rfmt provides three distinct interfaces for different use cases:

| Interface | Entry Point | Primary Use Case | Documentation |
| --- | --- | --- | --- |
| **CLI** | `exe/rfmt` | Batch formatting, CI/CD pipelines | [CLI Usage](https://deepwiki.com/fs0414/rfmt/3.1-cli-usage.html) |
| **Ruby API** | `Rfmt.format` | Programmatic integration, custom scripts | [Ruby API](https://deepwiki.com/fs0414/rfmt/3.2-ruby-api.html) |
| **Editor Integration** | `RubyLsp::Rfmt::FormatterRunner` | Format-on-save, interactive formatting | [Editor Integration](https://deepwiki.com/fs0414/rfmt/3.3-editor-integration.html) |

All three interfaces converge on the same core formatting engine but provide different levels of control and automation.

**Sources:** [README.md111-296](https://github.com/fs0414/rfmt/blob/90c575e6/README.md#L111-L296) [docs/user\_guide.md1-559](https://github.com/fs0414/rfmt/blob/90c575e6/docs/user_guide.md#L1-L559)

---

## User Interaction Flow

The following diagram illustrates how different user actions map to system components:

**Sources:** [README.md111-217](https://github.com/fs0414/rfmt/blob/90c575e6/README.md#L111-L217) [lib/rfmt/cli.rb1-400](https://github.com/fs0414/rfmt/blob/90c575e6/lib/rfmt/cli.rb#L1-L400) [docs/user\_guide.md44-84](https://github.com/fs0414/rfmt/blob/90c575e6/docs/user_guide.md#L44-L84)

---

## Quick Start Examples

### CLI Quick Start

The `exe/rfmt` executable provides the command-line interface:

**Exit codes** defined in [lib/rfmt/cli.rb25-28](https://github.com/fs0414/rfmt/blob/90c575e6/lib/rfmt/cli.rb#L25-L28):

* `EXIT_SUCCESS = 0` - All files formatted correctly
* `EXIT_ERROR = 1` - Error occurred during processing
* `EXIT_NEEDS_FORMAT = 2` - Files need formatting (check mode only)

**Sources:** [README.md155-187](https://github.com/fs0414/rfmt/blob/90c575e6/README.md#L155-L187) [docs/user\_guide.md45-84](https://github.com/fs0414/rfmt/blob/90c575e6/docs/user_guide.md#L45-L84)

### Ruby API Quick Start

The `Rfmt` module exposes programmatic formatting capabilities:

**Sources:** [README.md189-217](https://github.com/fs0414/rfmt/blob/90c575e6/README.md#L189-L217) [docs/user\_guide.md231-305](https://github.com/fs0414/rfmt/blob/90c575e6/docs/user_guide.md#L231-L305)

### Editor Integration Quick Start

**Neovim with autocmd:**

**VS Code with Ruby LSP:**

The `RubyLsp::Rfmt::FormatterRunner` class [lib/ruby\_lsp/rfmt.rb1-80](https://github.com/fs0414/rfmt/blob/90c575e6/lib/ruby_lsp/rfmt.rb#L1-L80) integrates with Ruby LSP for automatic format-on-save functionality.

**Sources:** [README.md277-296](https://github.com/fs0414/rfmt/blob/90c575e6/README.md#L277-L296) [docs/user\_guide.md10](https://github.com/fs0414/rfmt/blob/90c575e6/docs/user_guide.md#L10-L10)

---

## Configuration Workflow

Configuration discovery follows a hierarchical search pattern:

**Configuration file structure** [README.md121-143](https://github.com/fs0414/rfmt/blob/90c575e6/README.md#L121-L143):

**Sources:** [README.md113-248](https://github.com/fs0414/rfmt/blob/90c575e6/README.md#L113-L248) [docs/user\_guide.md86-173](https://github.com/fs0414/rfmt/blob/90c575e6/docs/user_guide.md#L86-L173) [lib/rfmt/config.rb1-150](https://github.com/fs0414/rfmt/blob/90c575e6/lib/rfmt/config.rb#L1-L150)

---

## Common Usage Patterns

### Pattern 1: Local Development Formatting

### Pattern 2: CI/CD Pipeline Integration

The `check` command uses `--check` flag internally [lib/rfmt/cli.rb200-250](https://github.com/fs0414/rfmt/blob/90c575e6/lib/rfmt/cli.rb#L200-L250) to verify formatting without modifications.

### Pattern 3: Pre-commit Hook Integration

**With Lefthook** [lefthook.yml](https://github.com/fs0414/rfmt/blob/90c575e6/lefthook.yml):

**With pre-commit framework:**

**Sources:** [docs/user\_guide.md496-520](https://github.com/fs0414/rfmt/blob/90c575e6/docs/user_guide.md#L496-L520)

### Pattern 4: Custom Script Integration

**Sources:** [docs/user\_guide.md231-271](https://github.com/fs0414/rfmt/blob/90c575e6/docs/user_guide.md#L231-L271)

---

## Data Flow Through Interfaces

This diagram shows how data flows through each interface to the formatting engine:

**Sources:** [lib/rfmt/cli.rb1-400](https://github.com/fs0414/rfmt/blob/90c575e6/lib/rfmt/cli.rb#L1-L400) [lib/rfmt.rb1-50](https://github.com/fs0414/rfmt/blob/90c575e6/lib/rfmt.rb#L1-L50) [lib/ruby\_lsp/rfmt.rb1-80](https://github.com/fs0414/rfmt/blob/90c575e6/lib/ruby_lsp/rfmt.rb#L1-L80)

---

## Configuration API Reference

The `Rfmt::Config` module [lib/rfmt/config.rb](https://github.com/fs0414/rfmt/blob/90c575e6/lib/rfmt/config.rb) provides configuration management:

| Method | Description | Returns | Example |
| --- | --- | --- | --- |
| `Config.init(path, force: false)` | Create config file | Boolean | `Rfmt::Config.init('.rfmt.yml')` |
| `Config.find` | Locate config file | String path or nil | `Rfmt::Config.find` |
| `Config.exists?` | Check if config exists | Boolean | `Rfmt::Config.exists?` |
| `Config.load(path = nil)` | Load configuration | Hash | `Rfmt::Config.load` |

**Usage example:**

**Sources:** [README.md229-248](https://github.com/fs0414/rfmt/blob/90c575e6/README.md#L229-L248) [docs/user\_guide.md86-128](https://github.com/fs0414/rfmt/blob/90c575e6/docs/user_guide.md#L86-L128)

---

## Debugging and Verbose Output

All interfaces support verbose output for troubleshooting:

### CLI Verbose Mode

Verbose output includes:

* Configuration file discovery [lib/rfmt/config.rb40-80](https://github.com/fs0414/rfmt/blob/90c575e6/lib/rfmt/config.rb#L40-L80)
* File processing steps [lib/rfmt/cli.rb120-180](https://github.com/fs0414/rfmt/blob/90c575e6/lib/rfmt/cli.rb#L120-L180)
* Cache hit/miss information [lib/rfmt/cache.rb50-100](https://github.com/fs0414/rfmt/blob/90c575e6/lib/rfmt/cache.rb#L50-L100)
* Rust extension operations [ext/rfmt/src/logging/logger.rs](https://github.com/fs0414/rfmt/blob/90c575e6/ext/rfmt/src/logging/logger.rs)

### API Verbose Mode

**Sources:** [README.md180-187](https://github.com/fs0414/rfmt/blob/90c575e6/README.md#L180-L187) [docs/user\_guide.md429-456](https://github.com/fs0414/rfmt/blob/90c575e6/docs/user_guide.md#L429-L456)

---

## Error Handling Across Interfaces

All interfaces raise consistent error types defined in [lib/rfmt/errors.rb](https://github.com/fs0414/rfmt/blob/90c575e6/lib/rfmt/errors.rb):

**CLI error handling** [lib/rfmt/cli.rb380-420](https://github.com/fs0414/rfmt/blob/90c575e6/lib/rfmt/cli.rb#L380-L420):

**API error handling:**

For detailed error codes and troubleshooting, see [Error Codes Reference](https://deepwiki.com/fs0414/rfmt/7.3-error-codes-reference.html).

**Sources:** [docs/user\_guide.md289-326](https://github.com/fs0414/rfmt/blob/90c575e6/docs/user_guide.md#L289-L326) [lib/rfmt/cli.rb380-420](https://github.com/fs0414/rfmt/blob/90c575e6/lib/rfmt/cli.rb#L380-L420)

---

## Performance Considerations

### Caching System

The CLI uses `Rfmt::Cache` [lib/rfmt/cache.rb](https://github.com/fs0414/rfmt/blob/90c575e6/lib/rfmt/cache.rb) to skip formatting of unchanged files:

Cache benefits:

* Reduces formatting time for large projects
* Tracks file hashes to detect changes
* Persists across runs in `.rfmt_cache` directory

### Parallel Processing

The CLI supports parallel file processing [lib/rfmt/cli.rb250-300](https://github.com/fs0414/rfmt/blob/90c575e6/lib/rfmt/cli.rb#L250-L300):

For performance benchmarks and optimization tips, see [Performance and Benchmarks](https://deepwiki.com/fs0414/rfmt/7.4-performance-and-benchmarks.html).

**Sources:** [README.md49-75](https://github.com/fs0414/rfmt/blob/90c575e6/README.md#L49-L75) [lib/rfmt/cache.rb1-200](https://github.com/fs0414/rfmt/blob/90c575e6/lib/rfmt/cache.rb#L1-L200)

---

## Summary

This guide covers the three primary usage patterns for rfmt:

1. **CLI Usage** ([detailed in 3.1](https://deepwiki.com/fs0414/rfmt/3.1-cli-usage.html)): Command-line tool for batch formatting and CI/CD integration
2. **Ruby API** ([detailed in 3.2](https://deepwiki.com/fs0414/rfmt/3.2-ruby-api.html)): Programmatic interface for custom scripts and integrations
3. **Editor Integration** ([detailed in 3.3](https://deepwiki.com/fs0414/rfmt/3.3-editor-integration.html)): Format-on-save functionality through Ruby LSP

All interfaces share:

* Common configuration system via `.rfmt.yml`
* Consistent error handling with error codes
* Same core formatting engine
* Verbose output for debugging

For implementation details of the formatting engine, see [Formatting Engine (Emitter)](https://deepwiki.com/fs0414/rfmt/4.2-formatting-engine-(emitter).html). For configuration options reference, see [Configuration Options Reference](https://deepwiki.com/fs0414/rfmt/7.2-configuration-options-reference.html).

**Sources:** [README.md1-398](https://github.com/fs0414/rfmt/blob/90c575e6/README.md#L1-L398) [docs/user\_guide.md1-559](https://github.com/fs0414/rfmt/blob/90c575e6/docs/user_guide.md#L1-L559)

Dismiss

Refresh this wiki

Enter email to refresh

### On this page

* [User Guides](https://deepwiki.com/fs0414/rfmt/3-user-guides.html#user-guides)
* [Overview of Usage Patterns](https://deepwiki.com/fs0414/rfmt/3-user-guides.html#overview-of-usage-patterns)
* [User Interaction Flow](https://deepwiki.com/fs0414/rfmt/3-user-guides.html#user-interaction-flow)
* [Quick Start Examples](https://deepwiki.com/fs0414/rfmt/3-user-guides.html#quick-start-examples)
* [CLI Quick Start](https://deepwiki.com/fs0414/rfmt/3-user-guides.html#cli-quick-start)
* [Ruby API Quick Start](https://deepwiki.com/fs0414/rfmt/3-user-guides.html#ruby-api-quick-start)
* [Editor Integration Quick Start](https://deepwiki.com/fs0414/rfmt/3-user-guides.html#editor-integration-quick-start)
* [Configuration Workflow](https://deepwiki.com/fs0414/rfmt/3-user-guides.html#configuration-workflow)
* [Common Usage Patterns](https://deepwiki.com/fs0414/rfmt/3-user-guides.html#common-usage-patterns)
* [Pattern 1: Local Development Formatting](https://deepwiki.com/fs0414/rfmt/3-user-guides.html#pattern-1-local-development-formatting)
* [Pattern 2: CI/CD Pipeline Integration](https://deepwiki.com/fs0414/rfmt/3-user-guides.html#pattern-2-cicd-pipeline-integration)
* [Pattern 3: Pre-commit Hook Integration](https://deepwiki.com/fs0414/rfmt/3-user-guides.html#pattern-3-pre-commit-hook-integration)
* [Pattern 4: Custom Script Integration](https://deepwiki.com/fs0414/rfmt/3-user-guides.html#pattern-4-custom-script-integration)
* [Data Flow Through Interfaces](https://deepwiki.com/fs0414/rfmt/3-user-guides.html#data-flow-through-interfaces)
* [Configuration API Reference](https://deepwiki.com/fs0414/rfmt/3-user-guides.html#configuration-api-reference)
* [Debugging and Verbose Output](https://deepwiki.com/fs0414/rfmt/3-user-guides.html#debugging-and-verbose-output)
* [CLI Verbose Mode](https://deepwiki.com/fs0414/rfmt/3-user-guides.html#cli-verbose-mode)
* [API Verbose Mode](https://deepwiki.com/fs0414/rfmt/3-user-guides.html#api-verbose-mode)
* [Error Handling Across Interfaces](https://deepwiki.com/fs0414/rfmt/3-user-guides.html#error-handling-across-interfaces)
* [Performance Considerations](https://deepwiki.com/fs0414/rfmt/3-user-guides.html#performance-considerations)
* [Caching System](https://deepwiki.com/fs0414/rfmt/3-user-guides.html#caching-system)
* [Parallel Processing](https://deepwiki.com/fs0414/rfmt/3-user-guides.html#parallel-processing)
* [Summary](https://deepwiki.com/fs0414/rfmt/3-user-guides.html#summary)
