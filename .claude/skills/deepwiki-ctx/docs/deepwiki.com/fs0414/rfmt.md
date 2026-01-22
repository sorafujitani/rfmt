---
title: "fs0414/rfmt | DeepWiki"
source_url: "https://deepwiki.com/fs0414/rfmt"
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

# Overview

Relevant source files

* [CHANGELOG.md](https://github.com/fs0414/rfmt/blob/90c575e6/CHANGELOG.md)
* [Gemfile.lock](https://github.com/fs0414/rfmt/blob/90c575e6/Gemfile.lock)
* [README.md](https://github.com/fs0414/rfmt/blob/90c575e6/README.md)
* [ext/rfmt/src/emitter/mod.rs](https://github.com/fs0414/rfmt/blob/90c575e6/ext/rfmt/src/emitter/mod.rs)
* [lib/rfmt/prism\_bridge.rb](https://github.com/fs0414/rfmt/blob/90c575e6/lib/rfmt/prism_bridge.rb)
* [lib/rfmt/prism\_node\_extractor.rb](https://github.com/fs0414/rfmt/blob/90c575e6/lib/rfmt/prism_node_extractor.rb)

This document provides a high-level introduction to rfmt, a Ruby code formatter with a Rust core. It covers the system's purpose, architecture, and key design decisions. For detailed information about specific subsystems, see:

* Architecture details: [Architecture Overview](https://deepwiki.com/fs0414/rfmt/1.1-architecture-overview.html)
* Core terminology and concepts: [Key Concepts](https://deepwiki.com/fs0414/rfmt/1.2-key-concepts.html)
* Installation and setup: [Getting Started](https://deepwiki.com/fs0414/rfmt/2-getting-started.html)
* Detailed component documentation: [Core Systems](https://deepwiki.com/fs0414/rfmt/4-core-systems.html)

## What is rfmt?

rfmt is a Ruby code formatter that transforms Ruby source code into a consistent, standardized format. It is designed to be:

* **Opinionated**: Enforces a consistent style with minimal configuration
* **Idempotent**: Produces identical output when run multiple times on the same input
* **Comment-preserving**: Maintains comment placement and formatting
* **Fast**: Core formatting engine implemented in Rust for performance

rfmt parses Ruby code using the Prism parser, converts the abstract syntax tree (AST) to an internal representation, applies formatting rules, and emits formatted Ruby code while preserving comments.

**Sources:** [README.md23-33](https://github.com/fs0414/rfmt/blob/90c575e6/README.md#L23-L33) [README.md34-48](https://github.com/fs0414/rfmt/blob/90c575e6/README.md#L34-L48)

## System Architecture

rfmt implements a hybrid Ruby-Rust architecture split across a language boundary. The Ruby layer handles parsing, configuration, and user interaction, while the Rust layer performs formatting operations.

**Diagram: Core Components and Language Boundary**

The system architecture consists of three distinct layers:

1. **Ruby Layer**: User-facing interfaces and integration with Ruby ecosystem
2. **FFI Boundary**: Data serialization and cross-language communication via Magnus
3. **Rust Layer**: Performance-critical formatting operations

**Sources:** [README.md32](https://github.com/fs0414/rfmt/blob/90c575e6/README.md#L32-L32) [ext/rfmt/src/emitter/mod.rs1-30](https://github.com/fs0414/rfmt/blob/90c575e6/ext/rfmt/src/emitter/mod.rs#L1-L30) [lib/rfmt/prism\_bridge.rb1-26](https://github.com/fs0414/rfmt/blob/90c575e6/lib/rfmt/prism_bridge.rb#L1-L26)

## Data Flow

The formatting process follows a linear pipeline from Ruby source to formatted output:

**Diagram: Data Flow from Source to Formatted Output**

The pipeline operates in distinct phases:

1. **Parsing Phase**: [lib/rfmt/prism\_bridge.rb20-26](https://github.com/fs0414/rfmt/blob/90c575e6/lib/rfmt/prism_bridge.rb#L20-L26) invokes `Prism.parse()` to generate a native Ruby AST
2. **Conversion Phase**: [lib/rfmt/prism\_bridge.rb87-98](https://github.com/fs0414/rfmt/blob/90c575e6/lib/rfmt/prism_bridge.rb#L87-L98) converts Prism nodes to rfmt's internal representation
3. **Serialization Phase**: [lib/rfmt/prism\_bridge.rb63-84](https://github.com/fs0414/rfmt/blob/90c575e6/lib/rfmt/prism_bridge.rb#L63-L84) serializes AST and comments to JSON
4. **FFI Transfer**: Magnus transfers the JSON payload across the language boundary
5. **Formatting Phase**: [ext/rfmt/src/emitter/mod.rs45-62](https://github.com/fs0414/rfmt/blob/90c575e6/ext/rfmt/src/emitter/mod.rs#L45-L62) emits formatted code
6. **Comment Reinsertion**: [ext/rfmt/src/emitter/mod.rs94-146](https://github.com/fs0414/rfmt/blob/90c575e6/ext/rfmt/src/emitter/mod.rs#L94-L146) reinserts comments at correct positions

**Sources:** [lib/rfmt/prism\_bridge.rb20-26](https://github.com/fs0414/rfmt/blob/90c575e6/lib/rfmt/prism_bridge.rb#L20-L26) [lib/rfmt/prism\_bridge.rb63-84](https://github.com/fs0414/rfmt/blob/90c575e6/lib/rfmt/prism_bridge.rb#L63-L84) [ext/rfmt/src/emitter/mod.rs45-62](https://github.com/fs0414/rfmt/blob/90c575e6/ext/rfmt/src/emitter/mod.rs#L45-L62)

## Entry Points

rfmt provides multiple interfaces for different use cases:

| Interface | Entry Point | Purpose | Primary File |
| --- | --- | --- | --- |
| **CLI** | `exe/rfmt` | Command-line formatting tool | [exe/rfmt](https://github.com/fs0414/rfmt/blob/90c575e6/exe/rfmt) |
| **Ruby API** | `Rfmt.format()` | Programmatic formatting | [lib/rfmt.rb](https://github.com/fs0414/rfmt/blob/90c575e6/lib/rfmt.rb) |
| **Editor Integration** | `RubyLsp::Rfmt::FormatterRunner` | LSP-based editor support | [lib/ruby\_lsp/rfmt/](https://github.com/fs0414/rfmt/blob/90c575e6/lib/ruby_lsp/rfmt/) |
| **Configuration** | `Rfmt::Config` | Configuration management | [lib/rfmt/config.rb](https://github.com/fs0414/rfmt/blob/90c575e6/lib/rfmt/config.rb) |

### CLI Entry Point

The command-line interface at [exe/rfmt](https://github.com/fs0414/rfmt/blob/90c575e6/exe/rfmt) processes files and directories:

**Diagram: CLI Command Flow**

The CLI supports these commands:

* `rfmt <files>`: Format specified files
* `rfmt check`: Verify formatting without modifications (exit code 0=formatted, 2=needs formatting)
* `rfmt init`: Generate `.rfmt.yml` configuration file
* `rfmt --diff`: Show formatting changes without applying them
* `rfmt --verbose`: Enable debug logging

**Sources:** [README.md156-187](https://github.com/fs0414/rfmt/blob/90c575e6/README.md#L156-L187) [README.md169-173](https://github.com/fs0414/rfmt/blob/90c575e6/README.md#L169-L173)

### Ruby API

The `Rfmt.format()` method provides programmatic access:

Example usage from [README.md193-217](https://github.com/fs0414/rfmt/blob/90c575e6/README.md#L193-L217):

**Sources:** [README.md189-217](https://github.com/fs0414/rfmt/blob/90c575e6/README.md#L189-L217)

## Key Design Decisions

### Hybrid Ruby-Rust Architecture

rfmt splits functionality across two languages to leverage their respective strengths:

| Aspect | Ruby | Rust |
| --- | --- | --- |
| **Parsing** | Prism parser (native Ruby) | N/A |
| **AST Serialization** | `Rfmt::PrismBridge` | N/A |
| **Configuration** | File discovery & loading | Rule application |
| **Formatting** | N/A | `Emitter` struct |
| **Performance** | Developer ergonomics | Computational speed |

The FFI boundary uses Magnus [ext/rfmt/Cargo.toml](https://github.com/fs0414/rfmt/blob/90c575e6/ext/rfmt/Cargo.toml) which provides low-overhead Ruby-Rust interop via `rb_sys`.

**Sources:** [Gemfile.lock4-5](https://github.com/fs0414/rfmt/blob/90c575e6/Gemfile.lock#L4-L5) [Gemfile.lock41-42](https://github.com/fs0414/rfmt/blob/90c575e6/Gemfile.lock#L41-L42)

### Comment Preservation

Comments are collected separately from the AST and reinserted during emission. The [ext/rfmt/src/emitter/mod.rs86-91](https://github.com/fs0414/rfmt/blob/90c575e6/ext/rfmt/src/emitter/mod.rs#L86-L91) `collect_comments()` function recursively extracts all comments, then [ext/rfmt/src/emitter/mod.rs94-123](https://github.com/fs0414/rfmt/blob/90c575e6/ext/rfmt/src/emitter/mod.rs#L94-L123) `emit_comments_before()` and [ext/rfmt/src/emitter/mod.rs126-146](https://github.com/fs0414/rfmt/blob/90c575e6/ext/rfmt/src/emitter/mod.rs#L126-L146) `emit_trailing_comments()` reinsert them at appropriate positions based on line numbers.

This approach ensures comments survive the formatting process and appear in semantically correct locations.

**Sources:** [ext/rfmt/src/emitter/mod.rs86-91](https://github.com/fs0414/rfmt/blob/90c575e6/ext/rfmt/src/emitter/mod.rs#L86-L91) [ext/rfmt/src/emitter/mod.rs94-146](https://github.com/fs0414/rfmt/blob/90c575e6/ext/rfmt/src/emitter/mod.rs#L94-L146)

### Source Extraction Fallback

For complex Ruby syntax that is difficult to reconstruct (e.g., method parameters with default values, lambda syntax), rfmt falls back to extracting the original source text. The [ext/rfmt/src/emitter/mod.rs34-42](https://github.com/fs0414/rfmt/blob/90c575e6/ext/rfmt/src/emitter/mod.rs#L34-L42) `Emitter::with_source()` constructor stores the original source, enabling methods like [ext/rfmt/src/emitter/mod.rs988-1001](https://github.com/fs0414/rfmt/blob/90c575e6/ext/rfmt/src/emitter/mod.rs#L988-L1001) `emit_generic_without_comments()` to extract text by offset.

This hybrid approach balances formatting correctness with implementation complexity.

**Sources:** [ext/rfmt/src/emitter/mod.rs34-42](https://github.com/fs0414/rfmt/blob/90c575e6/ext/rfmt/src/emitter/mod.rs#L34-L42) [ext/rfmt/src/emitter/mod.rs988-1001](https://github.com/fs0414/rfmt/blob/90c575e6/ext/rfmt/src/emitter/mod.rs#L988-L1001)

## Node Type System

rfmt defines 80+ node types representing Ruby language constructs. The [ext/rfmt/src/emitter/mod.rs149-171](https://github.com/fs0414/rfmt/blob/90c575e6/ext/rfmt/src/emitter/mod.rs#L149-L171) `emit_node()` function dispatches to specialized handlers:

**Diagram: Node Type Dispatch Pattern**

Common node categories:

* **Definitions**: `ClassNode`, `ModuleNode`, `DefNode`
* **Control Flow**: `IfNode`, `UnlessNode`, `CaseNode`, `WhileNode`, `UntilNode`, `ForNode`
* **Expressions**: `CallNode`, `LambdaNode`
* **Exception Handling**: `BeginNode`, `RescueNode`, `EnsureNode`
* **Literals**: `StringNode`, `IntegerNode`, `ArrayNode`, `HashNode`

Each specialized `emit_*` method understands the structure of its node type and formats accordingly.

**Sources:** [ext/rfmt/src/emitter/mod.rs149-171](https://github.com/fs0414/rfmt/blob/90c575e6/ext/rfmt/src/emitter/mod.rs#L149-L171) [ext/rfmt/src/emitter/mod.rs230-274](https://github.com/fs0414/rfmt/blob/90c575e6/ext/rfmt/src/emitter/mod.rs#L230-L274) [ext/rfmt/src/emitter/mod.rs316-385](https://github.com/fs0414/rfmt/blob/90c575e6/ext/rfmt/src/emitter/mod.rs#L316-L385)

## Configuration System

Configuration follows a hierarchical discovery pattern implemented in [lib/rfmt/config.rb](https://github.com/fs0414/rfmt/blob/90c575e6/lib/rfmt/config.rb) The system searches:

1. Current directory: `.rfmt.yml`, `.rfmt.yaml`, `rfmt.yml`, `rfmt.yaml`
2. Parent directories (walking up to root)
3. Home directory: `~/.rfmt.yml`, `~/.rfmt.yaml`, `rfmt.yml`, `rfmt.yaml`
4. Built-in defaults

Configuration structure from [README.md122-143](https://github.com/fs0414/rfmt/blob/90c575e6/README.md#L122-L143):

The Ruby layer loads configuration, while the Rust `Config` struct at [ext/rfmt/src/config/mod.rs](https://github.com/fs0414/rfmt/blob/90c575e6/ext/rfmt/src/config/mod.rs) applies formatting rules during emission.

**Sources:** [README.md122-143](https://github.com/fs0414/rfmt/blob/90c575e6/README.md#L122-L143) [README.md219-248](https://github.com/fs0414/rfmt/blob/90c575e6/README.md#L219-L248)

## Performance Characteristics

rfmt's Rust core provides consistent performance regardless of file count. Benchmark data from [README.md49-75](https://github.com/fs0414/rfmt/blob/90c575e6/README.md#L49-L75):

| Test Type | Files | rfmt | RuboCop | Ratio |
| --- | --- | --- | --- | --- |
| Single File | 1 | 191ms | 1.38s | 7.2x |
| Directory | 14 | 176ms | 1.68s | 9.6x |
| Full Project | 111 | 172ms | 4.36s | 25.4x |

Key observations:

* Execution time remains constant (172-191ms) across different file counts
* Low variance (8-23ms standard deviation)
* No linting overhead (formatting-only tool)

The cache system at [lib/rfmt/cache.rb](https://github.com/fs0414/rfmt/blob/90c575e6/lib/rfmt/cache.rb) uses SHA256 hashing to skip reformatting unchanged files.

**Sources:** [README.md49-75](https://github.com/fs0414/rfmt/blob/90c575e6/README.md#L49-L75)

## Extension and Customization

rfmt is designed with minimal configuration:

* No per-rule toggles (opinionated formatting)
* Limited configuration options (line length, indent width, quote style)
* Focus on consistency over flexibility

For code quality checks and linting, rfmt is intended to complement tools like RuboCop rather than replace them [README.md358-369](https://github.com/fs0414/rfmt/blob/90c575e6/README.md#L358-L369)

**Sources:** [README.md358-369](https://github.com/fs0414/rfmt/blob/90c575e6/README.md#L358-L369) [README.md27-33](https://github.com/fs0414/rfmt/blob/90c575e6/README.md#L27-L33)

Dismiss

Refresh this wiki

Enter email to refresh

### On this page

* [Overview](https://deepwiki.com/fs0414/rfmt.html#overview)
* [What is rfmt?](https://deepwiki.com/fs0414/rfmt.html#what-is-rfmt)
* [System Architecture](https://deepwiki.com/fs0414/rfmt.html#system-architecture)
* [Data Flow](https://deepwiki.com/fs0414/rfmt.html#data-flow)
* [Entry Points](https://deepwiki.com/fs0414/rfmt.html#entry-points)
* [CLI Entry Point](https://deepwiki.com/fs0414/rfmt.html#cli-entry-point)
* [Ruby API](https://deepwiki.com/fs0414/rfmt.html#ruby-api)
* [Key Design Decisions](https://deepwiki.com/fs0414/rfmt.html#key-design-decisions)
* [Hybrid Ruby-Rust Architecture](https://deepwiki.com/fs0414/rfmt.html#hybrid-ruby-rust-architecture)
* [Comment Preservation](https://deepwiki.com/fs0414/rfmt.html#comment-preservation)
* [Source Extraction Fallback](https://deepwiki.com/fs0414/rfmt.html#source-extraction-fallback)
* [Node Type System](https://deepwiki.com/fs0414/rfmt.html#node-type-system)
* [Configuration System](https://deepwiki.com/fs0414/rfmt.html#configuration-system)
* [Performance Characteristics](https://deepwiki.com/fs0414/rfmt.html#performance-characteristics)
* [Extension and Customization](https://deepwiki.com/fs0414/rfmt.html#extension-and-customization)
