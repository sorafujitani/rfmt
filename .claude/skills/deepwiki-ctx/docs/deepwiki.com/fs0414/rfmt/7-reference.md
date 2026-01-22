---
title: "Reference | fs0414/rfmt | DeepWiki"
source_url: "https://deepwiki.com/fs0414/rfmt/7-reference"
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

# Reference

Relevant source files

* [docs/user\_guide.ja.md](https://github.com/fs0414/rfmt/blob/90c575e6/docs/user_guide.ja.md)
* [docs/user\_guide.md](https://github.com/fs0414/rfmt/blob/90c575e6/docs/user_guide.md)
* [ext/rfmt/src/ast/mod.rs](https://github.com/fs0414/rfmt/blob/90c575e6/ext/rfmt/src/ast/mod.rs)

## Purpose and Scope

This section provides technical reference documentation for rfmt's data structures, configuration options, error codes, and performance characteristics. It serves as a quick lookup guide for developers integrating rfmt, troubleshooting issues, or contributing to the codebase.

For detailed information about specific topics:

* **Node Type Details**: See [Node Types Reference](https://deepwiki.com/fs0414/rfmt/7.1-node-types-reference.html) for complete enumeration of all AST node variants
* **Configuration Schema**: See [Configuration Options Reference](https://deepwiki.com/fs0414/rfmt/7.2-configuration-options-reference.html) for all available settings and validation rules
* **Error Documentation**: See [Error Codes Reference](https://deepwiki.com/fs0414/rfmt/7.3-error-codes-reference.html) for error code meanings and troubleshooting
* **Performance Analysis**: See [Performance and Benchmarks](https://deepwiki.com/fs0414/rfmt/7.4-performance-and-benchmarks.html) for detailed benchmarks and optimization tips

---

## AST Structure Overview

The core AST representation in rfmt is defined in the Rust layer and consists of several interconnected structures that capture Ruby's syntax tree with metadata necessary for formatting.

### AST Node Structure

**Sources:** [ext/rfmt/src/ast/mod.rs4-14](https://github.com/fs0414/rfmt/blob/90c575e6/ext/rfmt/src/ast/mod.rs#L4-L14) [ext/rfmt/src/ast/mod.rs16-25](https://github.com/fs0414/rfmt/blob/90c575e6/ext/rfmt/src/ast/mod.rs#L16-L25) [ext/rfmt/src/ast/mod.rs307-327](https://github.com/fs0414/rfmt/blob/90c575e6/ext/rfmt/src/ast/mod.rs#L307-L327) [ext/rfmt/src/ast/mod.rs329-338](https://github.com/fs0414/rfmt/blob/90c575e6/ext/rfmt/src/ast/mod.rs#L329-L338)

### Node Structure Fields

The `Node` struct is the fundamental building block of rfmt's AST representation:

| Field | Type | Purpose |
| --- | --- | --- |
| `node_type` | `NodeType` | Enum identifying the syntactic construct (class, method, call, etc.) |
| `location` | `Location` | Source position with line, column, and byte offsets |
| `children` | `Vec<Node>` | Ordered list of child nodes |
| `metadata` | `HashMap<String, String>` | Additional properties extracted by PrismNodeExtractor (names, counts, etc.) |
| `comments` | `Vec<Comment>` | Comments associated with this node |
| `formatting` | `FormattingInfo` | Layout hints and indentation tracking |

**Sources:** [ext/rfmt/src/ast/mod.rs6-14](https://github.com/fs0414/rfmt/blob/90c575e6/ext/rfmt/src/ast/mod.rs#L6-L14)

### Location Tracking

The `Location` struct provides precise source position information:

| Field | Type | Description |
| --- | --- | --- |
| `start_line` | `usize` | Starting line number (1-based) |
| `start_column` | `usize` | Starting column number (0-based) |
| `end_line` | `usize` | Ending line number (1-based) |
| `end_column` | `usize` | Ending column number (0-based) |
| `start_offset` | `usize` | Starting byte offset in source |
| `end_offset` | `usize` | Ending byte offset in source |

**Sources:** [ext/rfmt/src/ast/mod.rs17-25](https://github.com/fs0414/rfmt/blob/90c575e6/ext/rfmt/src/ast/mod.rs#L17-L25)

### Comment Preservation

Comments are tracked with type and position information to enable accurate preservation during formatting:

| Field | Type | Values |
| --- | --- | --- |
| `text` | `String` | Raw comment text including delimiters |
| `location` | `Location` | Position in source |
| `comment_type` | `CommentType` | `Line` (`#`) or `Block` (`=begin...=end`) |
| `position` | `CommentPosition` | `Leading`, `Trailing`, or `Inner` |

**Sources:** [ext/rfmt/src/ast/mod.rs308-314](https://github.com/fs0414/rfmt/blob/90c575e6/ext/rfmt/src/ast/mod.rs#L308-L314) [ext/rfmt/src/ast/mod.rs316-327](https://github.com/fs0414/rfmt/blob/90c575e6/ext/rfmt/src/ast/mod.rs#L316-L327)

---

## NodeType Categories

The `NodeType` enum contains over 80 variants categorized by syntactic purpose. For complete documentation, see [Node Types Reference](https://deepwiki.com/fs0414/rfmt/7.1-node-types-reference.html).

### Node Type Mapping Diagram

**Sources:** [ext/rfmt/src/ast/mod.rs28-184](https://github.com/fs0414/rfmt/blob/90c575e6/ext/rfmt/src/ast/mod.rs#L28-L184)

### Common Node Types Quick Reference

| Category | Node Types | Line References |
| --- | --- | --- |
| Program Structure | `ProgramNode`, `StatementsNode` | [ext/rfmt/src/ast/mod.rs31-33](https://github.com/fs0414/rfmt/blob/90c575e6/ext/rfmt/src/ast/mod.rs#L31-L33) |
| Definitions | `ClassNode`, `ModuleNode`, `DefNode` | [ext/rfmt/src/ast/mod.rs35-38](https://github.com/fs0414/rfmt/blob/90c575e6/ext/rfmt/src/ast/mod.rs#L35-L38) |
| Expressions | `CallNode`, `IfNode`, `UnlessNode` | [ext/rfmt/src/ast/mod.rs40-44](https://github.com/fs0414/rfmt/blob/90c575e6/ext/rfmt/src/ast/mod.rs#L40-L44) |
| Literals | `StringNode`, `IntegerNode`, `ArrayNode`, `HashNode` | [ext/rfmt/src/ast/mod.rs50-59](https://github.com/fs0414/rfmt/blob/90c575e6/ext/rfmt/src/ast/mod.rs#L50-L59) |
| Variables | Local, Instance, Class, Global variants | [ext/rfmt/src/ast/mod.rs72-132](https://github.com/fs0414/rfmt/blob/90c575e6/ext/rfmt/src/ast/mod.rs#L72-L132) |
| Control Flow | `WhileNode`, `UntilNode`, `ForNode`, `BreakNode`, `NextNode` | [ext/rfmt/src/ast/mod.rs94-106](https://github.com/fs0414/rfmt/blob/90c575e6/ext/rfmt/src/ast/mod.rs#L94-L106) |
| Exception Handling | `BeginNode`, `RescueNode`, `EnsureNode` | [ext/rfmt/src/ast/mod.rs46-49](https://github.com/fs0414/rfmt/blob/90c575e6/ext/rfmt/src/ast/mod.rs#L46-L49) [ext/rfmt/src/ast/mod.rs83-84](https://github.com/fs0414/rfmt/blob/90c575e6/ext/rfmt/src/ast/mod.rs#L83-L84) |
| Blocks | `BlockNode`, `LambdaNode` | [ext/rfmt/src/ast/mod.rs62](https://github.com/fs0414/rfmt/blob/90c575e6/ext/rfmt/src/ast/mod.rs#L62-L62) [ext/rfmt/src/ast/mod.rs79](https://github.com/fs0414/rfmt/blob/90c575e6/ext/rfmt/src/ast/mod.rs#L79-L79) |
| Pattern Matching | `CaseMatchNode`, `InNode`, `MatchPredicateNode` | [ext/rfmt/src/ast/mod.rs150-153](https://github.com/fs0414/rfmt/blob/90c575e6/ext/rfmt/src/ast/mod.rs#L150-L153) |

**Sources:** [ext/rfmt/src/ast/mod.rs28-184](https://github.com/fs0414/rfmt/blob/90c575e6/ext/rfmt/src/ast/mod.rs#L28-L184)

---

## Configuration Schema

Configuration is loaded from `.rfmt.yml` files and validated in both Ruby and Rust layers. For complete documentation, see [Configuration Options Reference](https://deepwiki.com/fs0414/rfmt/7.2-configuration-options-reference.html).

### Configuration File Discovery Flow

**Sources:** [docs/user\_guide.md120-128](https://github.com/fs0414/rfmt/blob/90c575e6/docs/user_guide.md#L120-L128)

### Configuration Structure

| Section | Options | Default Values |
| --- | --- | --- |
| **formatting** | Core formatting rules |  |
| `line_length` | Maximum line length | `100` |
| `indent_width` | Spaces per indent level | `2` |
| `indent_style` | `"spaces"` or `"tabs"` | `"spaces"` |
| `quote_style` | `"double"` or `"single"` | `"double"` |
| `hash_syntax` | Hash key syntax preference | `"ruby19"` |
| `trailing_comma` | Trailing comma rules | `"multiline"` |
| **include** | File patterns to format | `["**/*.rb", "**/*.rake"]` |
| **exclude** | File patterns to skip | `["vendor/**/*", "tmp/**/*"]` |
| **parser** | Parser-specific settings |  |

**Sources:** [docs/user\_guide.md89-118](https://github.com/fs0414/rfmt/blob/90c575e6/docs/user_guide.md#L89-L118) [docs/user\_guide.md130-173](https://github.com/fs0414/rfmt/blob/90c575e6/docs/user_guide.md#L130-L173)

### Validation Rules

| Option | Type | Validation | Error Code |
| --- | --- | --- | --- |
| `line_length` | Integer | Must be 40-500 | E002 |
| `indent_width` | Integer | Must be 1-8 | E002 |
| `indent_style` | String | Must be `"spaces"` or `"tabs"` | E002 |
| `quote_style` | String | Must be `"double"` or `"single"` | E002 |

**Sources:** [docs/user\_guide.md130-173](https://github.com/fs0414/rfmt/blob/90c575e6/docs/user_guide.md#L130-L173)

---

## Error Codes

rfmt uses structured error codes (E001-E999) for consistent error reporting. For complete documentation, see [Error Codes Reference](https://deepwiki.com/fs0414/rfmt/7.3-error-codes-reference.html).

### Error Code Categories

**Sources:** [docs/user\_guide.md311-326](https://github.com/fs0414/rfmt/blob/90c575e6/docs/user_guide.md#L311-L326) [docs/user\_guide.ja.md313-326](https://github.com/fs0414/rfmt/blob/90c575e6/docs/user_guide.ja.md#L313-L326)

### Common Error Codes Quick Reference

| Code | Type | Description | Typical Cause |
| --- | --- | --- | --- |
| **E001** | `ParseError` | Ruby syntax error in source code | Missing `end`, unclosed strings, invalid operators |
| **E002** | `ConfigError` | Invalid configuration file | Wrong types, invalid values, malformed YAML |
| **E003** | `IoError` | File read/write error | Permission denied, file not found, disk full |
| **E004** | `FormattingError` | Error during formatting process | Complex AST constructs, unexpected node types |
| **E005** | `RuleError` | Formatting rule application failed | Rule conflict, invalid transformation |
| **E006** | `UnsupportedFeature` | Feature not yet supported | Experimental Ruby syntax, edge cases |
| **E007** | `PrismError` | Prism parser integration error | Prism version mismatch, FFI failure |
| **E008** | `FormatError` | General formatting error | Generic formatting issues |
| **E999** | `InternalError` | Internal bug (should be reported) | Rust panics, unexpected state |

**Sources:** [docs/user\_guide.md315-326](https://github.com/fs0414/rfmt/blob/90c575e6/docs/user_guide.md#L315-L326) [docs/user\_guide.md344-388](https://github.com/fs0414/rfmt/blob/90c575e6/docs/user_guide.md#L344-L388)

### Error Message Format

All errors follow a consistent structure:

```
[Rfmt::<ErrorType>] <Description> in <file>:<line>:<column>
<Detailed message>

Code:
   <line-2> | <source>
   <line-1> | <source>
   <line>   | <source>
            |     ^
   <line+1> | <source>

Help: https://rfmt.dev/errors/<CODE>
```

**Sources:** [docs/user\_guide.md329-341](https://github.com/fs0414/rfmt/blob/90c575e6/docs/user_guide.md#L329-L341)

---

## Exit Codes

The `rfmt` CLI uses standard exit codes for programmatic integration:

| Exit Code | Meaning | Usage |
| --- | --- | --- |
| **0** | Success | All files formatted successfully or no changes needed |
| **1** | Error occurred | Parse errors, I/O errors, invalid configuration |
| **2** | Files need formatting | Used with `--check` flag for CI/CD validation |

**Sources:** [docs/user\_guide.md225-229](https://github.com/fs0414/rfmt/blob/90c575e6/docs/user_guide.md#L225-L229)

### CI/CD Integration Example

**Sources:** [docs/user\_guide.md72-78](https://github.com/fs0414/rfmt/blob/90c575e6/docs/user_guide.md#L72-L78) [docs/user\_guide.md225-229](https://github.com/fs0414/rfmt/blob/90c575e6/docs/user_guide.md#L225-L229)

---

## API Surface

### Ruby API Entry Points

| Method | Purpose | Return Type |
| --- | --- | --- |
| `Rfmt.format(source, config: {})` | Format Ruby source code | `String` |
| `Rfmt::PrismBridge.parse(source)` | Parse to JSON AST | `String` (JSON) |
| `Rfmt::Config.load(path)` | Load configuration file | `Hash` |
| `Rfmt::Config.init(path)` | Create default config file | `Boolean` |
| `Rfmt::Config.exists?(path)` | Check if config exists | `Boolean` |
| `Rfmt::Cache.load` | Load cache database | `Rfmt::Cache` |
| `Rfmt::Cache#formatted?(path)` | Check if file is cached | `Boolean` |
| `Rfmt::Cache#mark_formatted(path, hash)` | Update cache entry | `void` |

**Sources:** [docs/user\_guide.md233-305](https://github.com/fs0414/rfmt/blob/90c575e6/docs/user_guide.md#L233-L305)

### Rust FFI Interface

The Rust extension exposes formatted code through Magnus FFI:

| Function | Location | Purpose |
| --- | --- | --- |
| `format_ruby_code` | Magnus binding | Main formatting entry point |
| `Node::from_json` | Deserialization | Parse JSON AST into Rust structures |
| `Emitter::emit` | Formatting engine | Generate formatted output |
| `Config::from_json` | Configuration | Parse config from Ruby layer |

---

## Performance Characteristics

For detailed benchmarks and optimization strategies, see [Performance and Benchmarks](https://deepwiki.com/fs0414/rfmt/7.4-performance-and-benchmarks.html).

### Key Performance Factors

| Factor | Impact | Typical Value |
| --- | --- | --- |
| **File Size** | Linear scaling | ~1ms per 1000 lines |
| **AST Complexity** | Moderate impact | ~2ms for deeply nested code |
| **Cache Hit** | Skip processing | ~0.1ms file hash check |
| **FFI Overhead** | Per-file fixed cost | ~0.5ms serialization |
| **Comment Count** | Linear with comments | ~0.01ms per comment |

### Cache System

The cache system uses SHA-256 file hashing to skip formatting of unchanged files:

| Operation | Time Complexity | Space Complexity |
| --- | --- | --- |
| Hash calculation | O(n) file size | O(1) |
| Cache lookup | O(1) hash map | O(m) cached files |
| Cache update | O(1) insertion | O(1) per entry |

**Sources:** Based on caching architecture (detailed in page [4.5](https://deepwiki.com/fs0414/rfmt/4.5-caching-system.html))

---

## Supported Ruby Versions

| Ruby Version | Support Status | Notes |
| --- | --- | --- |
| **3.0** | ✅ Supported | Minimum version |
| **3.1** | ✅ Supported | Tested in CI |
| **3.2** | ✅ Supported | Tested in CI |
| **3.3** | ✅ Supported | Latest tested version |
| **2.x** | ❌ Not supported | Use Ruby 3.0+ |

**Sources:** [docs/user\_guide.md523-530](https://github.com/fs0414/rfmt/blob/90c575e6/docs/user_guide.md#L523-L530)

---

## Platform Support

rfmt provides pre-compiled native gems for common platforms:

| Platform | Architecture | Status |
| --- | --- | --- |
| **Linux** | x86\_64-linux | ✅ Pre-compiled gem |
| **Linux** | aarch64-linux | ✅ Pre-compiled gem |
| **macOS** | x86\_64-darwin | ✅ Pre-compiled gem |
| **macOS** | arm64-darwin | ✅ Pre-compiled gem |
| **Windows** | x64-mingw32 | ✅ Pre-compiled gem |

For unlisted platforms, rfmt requires Rust 1.70+ to compile from source.

**Sources:** Based on build system (detailed in page [6.1](https://deepwiki.com/fs0414/rfmt/6.1-building-from-source.html))

Dismiss

Refresh this wiki

Enter email to refresh

### On this page

* [Reference](https://deepwiki.com/fs0414/rfmt/7-reference.html#reference)
* [Purpose and Scope](https://deepwiki.com/fs0414/rfmt/7-reference.html#purpose-and-scope)
* [AST Structure Overview](https://deepwiki.com/fs0414/rfmt/7-reference.html#ast-structure-overview)
* [AST Node Structure](https://deepwiki.com/fs0414/rfmt/7-reference.html#ast-node-structure)
* [Node Structure Fields](https://deepwiki.com/fs0414/rfmt/7-reference.html#node-structure-fields)
* [Location Tracking](https://deepwiki.com/fs0414/rfmt/7-reference.html#location-tracking)
* [Comment Preservation](https://deepwiki.com/fs0414/rfmt/7-reference.html#comment-preservation)
* [NodeType Categories](https://deepwiki.com/fs0414/rfmt/7-reference.html#nodetype-categories)
* [Node Type Mapping Diagram](https://deepwiki.com/fs0414/rfmt/7-reference.html#node-type-mapping-diagram)
* [Common Node Types Quick Reference](https://deepwiki.com/fs0414/rfmt/7-reference.html#common-node-types-quick-reference)
* [Configuration Schema](https://deepwiki.com/fs0414/rfmt/7-reference.html#configuration-schema)
* [Configuration File Discovery Flow](https://deepwiki.com/fs0414/rfmt/7-reference.html#configuration-file-discovery-flow)
* [Configuration Structure](https://deepwiki.com/fs0414/rfmt/7-reference.html#configuration-structure)
* [Validation Rules](https://deepwiki.com/fs0414/rfmt/7-reference.html#validation-rules)
* [Error Codes](https://deepwiki.com/fs0414/rfmt/7-reference.html#error-codes)
* [Error Code Categories](https://deepwiki.com/fs0414/rfmt/7-reference.html#error-code-categories)
* [Common Error Codes Quick Reference](https://deepwiki.com/fs0414/rfmt/7-reference.html#common-error-codes-quick-reference)
* [Error Message Format](https://deepwiki.com/fs0414/rfmt/7-reference.html#error-message-format)
* [Exit Codes](https://deepwiki.com/fs0414/rfmt/7-reference.html#exit-codes)
* [CI/CD Integration Example](https://deepwiki.com/fs0414/rfmt/7-reference.html#cicd-integration-example)
* [API Surface](https://deepwiki.com/fs0414/rfmt/7-reference.html#api-surface)
* [Ruby API Entry Points](https://deepwiki.com/fs0414/rfmt/7-reference.html#ruby-api-entry-points)
* [Rust FFI Interface](https://deepwiki.com/fs0414/rfmt/7-reference.html#rust-ffi-interface)
* [Performance Characteristics](https://deepwiki.com/fs0414/rfmt/7-reference.html#performance-characteristics)
* [Key Performance Factors](https://deepwiki.com/fs0414/rfmt/7-reference.html#key-performance-factors)
* [Cache System](https://deepwiki.com/fs0414/rfmt/7-reference.html#cache-system)
* [Supported Ruby Versions](https://deepwiki.com/fs0414/rfmt/7-reference.html#supported-ruby-versions)
* [Platform Support](https://deepwiki.com/fs0414/rfmt/7-reference.html#platform-support)
