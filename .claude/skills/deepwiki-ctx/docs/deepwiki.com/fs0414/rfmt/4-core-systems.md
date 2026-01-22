---
title: "Core Systems | fs0414/rfmt | DeepWiki"
source_url: "https://deepwiki.com/fs0414/rfmt/4-core-systems"
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

# Core Systems

Relevant source files

* [ext/rfmt/src/ast/mod.rs](https://github.com/fs0414/rfmt/blob/90c575e6/ext/rfmt/src/ast/mod.rs)
* [ext/rfmt/src/emitter/mod.rs](https://github.com/fs0414/rfmt/blob/90c575e6/ext/rfmt/src/emitter/mod.rs)
* [lib/rfmt/prism\_bridge.rb](https://github.com/fs0414/rfmt/blob/90c575e6/lib/rfmt/prism_bridge.rb)
* [lib/rfmt/prism\_node\_extractor.rb](https://github.com/fs0414/rfmt/blob/90c575e6/lib/rfmt/prism_node_extractor.rb)

This document provides deep technical documentation of rfmt's internal architecture and components. It covers the core systems that power the formatter: AST representation, parser integration, formatting engine, configuration management, caching, and logging. Each subsystem is introduced here with architectural diagrams and then explored in detail in dedicated pages.

For user-facing documentation on how to use rfmt, see [User Guides](https://deepwiki.com/fs0414/rfmt/3-user-guides.html). For information about building and contributing, see [Development](https://deepwiki.com/fs0414/rfmt/6-development.html).

## System Architecture

The following diagram shows the primary components of rfmt's core systems and their relationships:

**Core System Components and Relationships**

**Sources:** [ext/rfmt/src/ast/mod.rs1-526](https://github.com/fs0414/rfmt/blob/90c575e6/ext/rfmt/src/ast/mod.rs#L1-L526) [ext/rfmt/src/emitter/mod.rs1-1174](https://github.com/fs0414/rfmt/blob/90c575e6/ext/rfmt/src/emitter/mod.rs#L1-L1174) [lib/rfmt/prism\_bridge.rb1-391](https://github.com/fs0414/rfmt/blob/90c575e6/lib/rfmt/prism_bridge.rb#L1-L391)

## AST Representation

The Abstract Syntax Tree (AST) is the central data structure that rfmt operates on. It bridges Ruby's Prism parser output to Rust's formatting engine.

### Node Structure

The `Node` struct is the fundamental building block of the AST:

**AST Node Components**

| Component | Type | Purpose |
| --- | --- | --- |
| `node_type` | `NodeType` | Identifies the syntactic construct (class, method, if, etc.) |
| `location` | `Location` | Line/column/offset positioning in source |
| `children` | `Vec<Node>` | Child nodes forming the tree structure |
| `metadata` | `HashMap<String, String>` | Node-specific data (names, parameter counts, etc.) |
| `comments` | `Vec<Comment>` | Comments associated with this node |
| `formatting` | `FormattingInfo` | Formatting hints (indentation, blank lines, etc.) |

**Sources:** [ext/rfmt/src/ast/mod.rs4-14](https://github.com/fs0414/rfmt/blob/90c575e6/ext/rfmt/src/ast/mod.rs#L4-L14)

### NodeType Taxonomy

The `NodeType` enum categorizes 80+ Ruby syntax constructs. The following table organizes them by semantic category:

| Category | Node Types | Example Usage |
| --- | --- | --- |
| **Structure** | `ProgramNode`, `StatementsNode` | Root and statement blocks |
| **Definitions** | `ClassNode`, `ModuleNode`, `DefNode` | Class, module, and method definitions |
| **Control Flow** | `IfNode`, `UnlessNode`, `CaseNode`, `WhenNode` | Conditionals and case statements |
| **Loops** | `WhileNode`, `UntilNode`, `ForNode` | Loop constructs |
| **Exception Handling** | `BeginNode`, `RescueNode`, `EnsureNode` | Exception management |
| **Expressions** | `CallNode`, `LambdaNode`, `BlockNode` | Method calls, lambdas, blocks |
| **Literals** | `StringNode`, `IntegerNode`, `ArrayNode`, `HashNode` | Literal values |
| **Variables** | `LocalVariableReadNode`, `InstanceVariableReadNode`, `ClassVariableReadNode`, `GlobalVariableReadNode` | Variable access |
| **Operators** | `OrNode`, `AndNode`, `NotNode`, `RangeNode` | Logical and range operators |
| **Assignments** | `LocalVariableWriteNode`, `InstanceVariableWriteNode`, `*OrWriteNode`, `*AndWriteNode`, `*OperatorWriteNode` | Variable assignments and compound assignments |

**Sources:** [ext/rfmt/src/ast/mod.rs28-184](https://github.com/fs0414/rfmt/blob/90c575e6/ext/rfmt/src/ast/mod.rs#L28-L184)

### Location Tracking

The `Location` struct provides precise positioning information for every node:

This dual representation (line/column + byte offsets) enables both human-readable error messages and efficient source extraction.

**Sources:** [ext/rfmt/src/ast/mod.rs16-25](https://github.com/fs0414/rfmt/blob/90c575e6/ext/rfmt/src/ast/mod.rs#L16-L25)

### Comment Representation

Comments are first-class citizens in rfmt's AST, ensuring they are preserved during formatting:

**Sources:** [ext/rfmt/src/ast/mod.rs307-327](https://github.com/fs0414/rfmt/blob/90c575e6/ext/rfmt/src/ast/mod.rs#L307-L327)

## Data Flow Pipeline

The following diagram illustrates how Ruby source code flows through rfmt's processing pipeline:

**Processing Pipeline: Source to Formatted Output**

**Sources:** [lib/rfmt/prism\_bridge.rb20-98](https://github.com/fs0414/rfmt/blob/90c575e6/lib/rfmt/prism_bridge.rb#L20-L98) [ext/rfmt/src/emitter/mod.rs44-62](https://github.com/fs0414/rfmt/blob/90c575e6/ext/rfmt/src/emitter/mod.rs#L44-L62) [ext/rfmt/src/emitter/mod.rs149-171](https://github.com/fs0414/rfmt/blob/90c575e6/ext/rfmt/src/emitter/mod.rs#L149-L171)

## Component Overview

The core systems are organized into six primary subsystems, each documented in detail on dedicated pages:

### 4.1 AST Representation

Detailed documentation of the `Node` structure, the complete `NodeType` enum with all 80+ variants, `Location` tracking, `Comment` representation, `FormattingInfo`, and serialization/deserialization across the FFI boundary.

**Key Types:**

* `ast::Node` - Primary AST node structure [ext/rfmt/src/ast/mod.rs4-14](https://github.com/fs0414/rfmt/blob/90c575e6/ext/rfmt/src/ast/mod.rs#L4-L14)
* `ast::NodeType` - Enum with 80+ variants [ext/rfmt/src/ast/mod.rs28-184](https://github.com/fs0414/rfmt/blob/90c575e6/ext/rfmt/src/ast/mod.rs#L28-L184)
* `ast::Location` - Position tracking [ext/rfmt/src/ast/mod.rs16-25](https://github.com/fs0414/rfmt/blob/90c575e6/ext/rfmt/src/ast/mod.rs#L16-L25)
* `ast::Comment` - Comment preservation [ext/rfmt/src/ast/mod.rs307-327](https://github.com/fs0414/rfmt/blob/90c575e6/ext/rfmt/src/ast/mod.rs#L307-L327)

See [AST Representation](https://deepwiki.com/fs0414/rfmt/4.1-ast-representation.html) for complete details.

### 4.2 Formatting Engine (Emitter)

In-depth explanation of the `Emitter` architecture: node dispatch pattern, specialized `emit_*` methods for different constructs, comment collection and reinsertion, source extraction strategy for complex syntax, and indentation management.

**Key Components:**

* `emitter::Emitter` - Main formatting engine [ext/rfmt/src/emitter/mod.rs13-30](https://github.com/fs0414/rfmt/blob/90c575e6/ext/rfmt/src/emitter/mod.rs#L13-L30)
* `emit_node()` - Node type dispatcher [ext/rfmt/src/emitter/mod.rs149-171](https://github.com/fs0414/rfmt/blob/90c575e6/ext/rfmt/src/emitter/mod.rs#L149-L171)
* `emit_class()`, `emit_method()`, etc. - Specialized formatters [ext/rfmt/src/emitter/mod.rs230-385](https://github.com/fs0414/rfmt/blob/90c575e6/ext/rfmt/src/emitter/mod.rs#L230-L385)
* `emit_generic()` - Source extraction fallback [ext/rfmt/src/emitter/mod.rs1005-1033](https://github.com/fs0414/rfmt/blob/90c575e6/ext/rfmt/src/emitter/mod.rs#L1005-L1033)
* `collect_comments()` - Comment gathering [ext/rfmt/src/emitter/mod.rs86-91](https://github.com/fs0414/rfmt/blob/90c575e6/ext/rfmt/src/emitter/mod.rs#L86-L91)

See [Formatting Engine (Emitter)](https://deepwiki.com/fs0414/rfmt/4.2-formatting-engine-(emitter).html) for complete details.

### 4.3 Parser Integration (PrismBridge)

Explains how `PrismBridge` interfaces with Ruby's Prism parser, converts Prism AST to rfmt's internal representation, extracts metadata (names, parameter counts), and serializes to JSON for FFI transfer.

**Key Components:**

* `Rfmt::PrismBridge` - Ruby-side parser integration [lib/rfmt/prism\_bridge.rb8-389](https://github.com/fs0414/rfmt/blob/90c575e6/lib/rfmt/prism_bridge.rb#L8-L389)
* `parse()` - Entry point [lib/rfmt/prism\_bridge.rb20-26](https://github.com/fs0414/rfmt/blob/90c575e6/lib/rfmt/prism_bridge.rb#L20-L26)
* `convert_node()` - AST conversion [lib/rfmt/prism\_bridge.rb87-98](https://github.com/fs0414/rfmt/blob/90c575e6/lib/rfmt/prism_bridge.rb#L87-L98)
* `extract_children()` - Child traversal [lib/rfmt/prism\_bridge.rb122-307](https://github.com/fs0414/rfmt/blob/90c575e6/lib/rfmt/prism_bridge.rb#L122-L307)
* `Rfmt::PrismNodeExtractor` - Metadata extraction helpers [lib/rfmt/prism\_node\_extractor.rb1-116](https://github.com/fs0414/rfmt/blob/90c575e6/lib/rfmt/prism_node_extractor.rb#L1-L116)

See [Parser Integration (PrismBridge)](https://deepwiki.com/fs0414/rfmt/4.3-parser-integration-(prismbridge).html) for complete details.

### 4.4 Configuration System

Documents the dual Ruby-Rust configuration architecture: discovery and loading in Ruby, validation, application in Rust, file filtering with include/exclude globs, and the `Config` struct hierarchy.

**Key Components:**

* `Rfmt::Config` - Ruby-side config management
* `config::Config` - Rust-side config application
* Configuration discovery (`.rfmt.yml` search from current directory upward)
* Validation rules (line\_length: 40-500, indent\_width: 1-8)

See [Configuration System](https://deepwiki.com/fs0414/rfmt/4.4-configuration-system.html) for complete details.

### 4.5 Caching System

Explains the cache mechanism (file hash comparison, timestamp tracking), cache lifecycle (`load`, `mark_formatted`, `save`), management commands (clear, stats, prune), and performance benefits.

**Key Components:**

* `Rfmt::Cache` - Cache implementation
* File hash computation for change detection
* Cache invalidation strategies

See [Caching System](https://deepwiki.com/fs0414/rfmt/4.5-caching-system.html) for complete details.

### 4.6 Logging System

Documents the `RfmtLogger` implementation, log level configuration (`DEBUG`, `RFMT_DEBUG`, `RFMT_LOG` environment variables), integration with Rust's log facade, and debug output patterns.

**Key Components:**

* `logging::RfmtLogger` - Rust logger implementation
* Environment variable configuration
* Integration with Ruby's logging

See [Logging System](https://deepwiki.com/fs0414/rfmt/4.6-logging-system.html) for complete details.

## Dispatch and Specialization Pattern

The `Emitter` uses a dispatch pattern to route different node types to specialized formatting methods. This design balances generality with syntax-specific knowledge:

**Emitter Dispatch Pattern**

**Sources:** [ext/rfmt/src/emitter/mod.rs149-171](https://github.com/fs0414/rfmt/blob/90c575e6/ext/rfmt/src/emitter/mod.rs#L149-L171)

### Specialized vs Generic Formatting

The system uses two strategies:

1. **Specialized Formatting**: For well-understood constructs (classes, methods, conditionals), rfmt reconstructs the syntax from the AST structure
2. **Generic Formatting (Source Extraction)**: For complex syntax (lambda parameters, method signatures with defaults), rfmt extracts the original source text

This hybrid approach ensures correctness while keeping the implementation maintainable.

**Specialized Methods:**

| Method | Node Types | Lines |
| --- | --- | --- |
| `emit_class()` | `ClassNode` | [ext/rfmt/src/emitter/mod.rs230-274](https://github.com/fs0414/rfmt/blob/90c575e6/ext/rfmt/src/emitter/mod.rs#L230-L274) |
| `emit_module()` | `ModuleNode` | [ext/rfmt/src/emitter/mod.rs277-313](https://github.com/fs0414/rfmt/blob/90c575e6/ext/rfmt/src/emitter/mod.rs#L277-L313) |
| `emit_method()` | `DefNode` | [ext/rfmt/src/emitter/mod.rs316-385](https://github.com/fs0414/rfmt/blob/90c575e6/ext/rfmt/src/emitter/mod.rs#L316-L385) |
| `emit_if_unless()` | `IfNode`, `UnlessNode` | [ext/rfmt/src/emitter/mod.rs650-823](https://github.com/fs0414/rfmt/blob/90c575e6/ext/rfmt/src/emitter/mod.rs#L650-L823) |
| `emit_call()` | `CallNode` | [ext/rfmt/src/emitter/mod.rs826-857](https://github.com/fs0414/rfmt/blob/90c575e6/ext/rfmt/src/emitter/mod.rs#L826-L857) |
| `emit_case()` | `CaseNode` | [ext/rfmt/src/emitter/mod.rs550-605](https://github.com/fs0414/rfmt/blob/90c575e6/ext/rfmt/src/emitter/mod.rs#L550-L605) |
| `emit_rescue()` | `RescueNode` | [ext/rfmt/src/emitter/mod.rs427-514](https://github.com/fs0414/rfmt/blob/90c575e6/ext/rfmt/src/emitter/mod.rs#L427-L514) |
| `emit_while_until()` | `WhileNode`, `UntilNode` | [ext/rfmt/src/emitter/mod.rs1047-1096](https://github.com/fs0414/rfmt/blob/90c575e6/ext/rfmt/src/emitter/mod.rs#L1047-L1096) |

**Generic Fallback:**

| Method | Purpose | Lines |
| --- | --- | --- |
| `emit_generic()` | Extract node from source, mark embedded comments as emitted | [ext/rfmt/src/emitter/mod.rs1005-1033](https://github.com/fs0414/rfmt/blob/90c575e6/ext/rfmt/src/emitter/mod.rs#L1005-L1033) |
| `emit_generic_without_comments()` | Extract without comment handling | [ext/rfmt/src/emitter/mod.rs988-1002](https://github.com/fs0414/rfmt/blob/90c575e6/ext/rfmt/src/emitter/mod.rs#L988-L1002) |

**Sources:** [ext/rfmt/src/emitter/mod.rs149-1174](https://github.com/fs0414/rfmt/blob/90c575e6/ext/rfmt/src/emitter/mod.rs#L149-L1174)

## Comment Preservation Strategy

Comment preservation is a critical feature that distinguishes rfmt from simpler formatters. The system collects all comments upfront, then strategically re-emits them at appropriate positions:

**Comment Processing Workflow**

**Key Methods:**

* `collect_comments()` - Recursively gathers all comments [ext/rfmt/src/emitter/mod.rs86-91](https://github.com/fs0414/rfmt/blob/90c575e6/ext/rfmt/src/emitter/mod.rs#L86-L91)
* `emit_comments_before()` - Emits comments before a line [ext/rfmt/src/emitter/mod.rs94-123](https://github.com/fs0414/rfmt/blob/90c575e6/ext/rfmt/src/emitter/mod.rs#L94-L123)
* `emit_trailing_comments()` - Emits comments on same line [ext/rfmt/src/emitter/mod.rs126-146](https://github.com/fs0414/rfmt/blob/90c575e6/ext/rfmt/src/emitter/mod.rs#L126-L146)
* `emit_remaining_comments()` - Emits any orphaned comments [ext/rfmt/src/emitter/mod.rs65-83](https://github.com/fs0414/rfmt/blob/90c575e6/ext/rfmt/src/emitter/mod.rs#L65-L83)

**Sources:** [ext/rfmt/src/emitter/mod.rs65-146](https://github.com/fs0414/rfmt/blob/90c575e6/ext/rfmt/src/emitter/mod.rs#L65-L146)

## Metadata Extraction

The `PrismNodeExtractor` module provides safe methods to extract metadata from Prism nodes. This encapsulates the logic for accessing Prism node properties, making the codebase resilient to Prism API changes:

| Extractor Method | Purpose | Used For |
| --- | --- | --- |
| `extract_node_name()` | Get node's name attribute | Method names, variable names |
| `extract_class_or_module_name()` | Get full class/module name | Handle namespaced names (Foo::Bar::Baz) |
| `extract_superclass_name()` | Get superclass name | Class inheritance |
| `extract_parameter_count()` | Count method parameters | Determine if method has parameters |
| `extract_message_name()` | Get method call message | Call node message |
| `extract_string_content()` | Get string literal value | String node content |
| `extract_literal_value()` | Get literal value | Integer, float, symbol values |

These extractors are used during AST conversion to populate the `metadata` HashMap in each `Node`.

**Sources:** [lib/rfmt/prism\_node\_extractor.rb1-116](https://github.com/fs0414/rfmt/blob/90c575e6/lib/rfmt/prism_node_extractor.rb#L1-L116) [lib/rfmt/prism\_bridge.rb310-368](https://github.com/fs0414/rfmt/blob/90c575e6/lib/rfmt/prism_bridge.rb#L310-L368)

## Indentation Management

The `Emitter` tracks indentation level throughout the formatting process and applies it consistently:

Each `emit_*` method receives an `indent_level` parameter and increments it when emitting nested structures:

* `emit_class()` calls child nodes with `indent_level + 1`
* `emit_method()` calls body statements with `indent_level + 1`
* `emit_if_unless()` indents then/else branches by 1 level

**Sources:** [ext/rfmt/src/emitter/mod.rs1036-1044](https://github.com/fs0414/rfmt/blob/90c575e6/ext/rfmt/src/emitter/mod.rs#L1036-L1044)

## Block Style Detection

For `CallNode` with blocks, the `Emitter` detects whether the original source used `do...end` or `{ }` style and preserves it:

This preserves the semantic distinction between multi-line blocks (`do...end`) and single-line blocks (`{ }`), which is a Ruby style convention.

**Sources:** [ext/rfmt/src/emitter/mod.rs860-873](https://github.com/fs0414/rfmt/blob/90c575e6/ext/rfmt/src/emitter/mod.rs#L860-L873) [ext/rfmt/src/emitter/mod.rs898-959](https://github.com/fs0414/rfmt/blob/90c575e6/ext/rfmt/src/emitter/mod.rs#L898-L959)

## Structural Node Filtering

Some nodes in the AST are "structural" - they're part of the definition syntax itself rather than the body. The `Emitter` filters these when emitting class/module/method bodies:

For example, in a class definition, the class name (`ConstantReadNode`) and superclass are structural - they're emitted as part of the `class Foo < Bar` line, not as separate statements in the body.

**Sources:** [ext/rfmt/src/emitter/mod.rs1151-1166](https://github.com/fs0414/rfmt/blob/90c575e6/ext/rfmt/src/emitter/mod.rs#L1151-L1166)

## Summary

The core systems of rfmt work together to transform Ruby source code into formatted output:

1. **PrismBridge** parses Ruby using Prism and converts to rfmt's internal AST representation
2. **AST structures** (Node, NodeType, Location, Comment) provide a rich representation
3. **JSON serialization** crosses the FFI boundary from Ruby to Rust
4. **Emitter** dispatches to specialized formatters or generic source extraction
5. **Comment preservation** ensures all comments are retained at correct positions
6. **Configuration** controls formatting rules (indentation, line length, etc.)
7. **Caching** avoids re-formatting unchanged files
8. **Logging** provides visibility into the formatting process

Each subsystem is documented in detail in the following pages: [AST Representation](https://deepwiki.com/fs0414/rfmt/4.1-ast-representation.html), [Formatting Engine](https://deepwiki.com/fs0414/rfmt/4.2-formatting-engine-(emitter).html), [Parser Integration](https://deepwiki.com/fs0414/rfmt/4.3-parser-integration-(prismbridge).html), [Configuration System](https://deepwiki.com/fs0414/rfmt/4.4-configuration-system.html), [Caching System](https://deepwiki.com/fs0414/rfmt/4.5-caching-system.html), and [Logging System](https://deepwiki.com/fs0414/rfmt/4.6-logging-system.html).

Dismiss

Refresh this wiki

Enter email to refresh

### On this page

* [Core Systems](https://deepwiki.com/fs0414/rfmt/4-core-systems.html#core-systems)
* [System Architecture](https://deepwiki.com/fs0414/rfmt/4-core-systems.html#system-architecture)
* [AST Representation](https://deepwiki.com/fs0414/rfmt/4-core-systems.html#ast-representation)
* [Node Structure](https://deepwiki.com/fs0414/rfmt/4-core-systems.html#node-structure)
* [NodeType Taxonomy](https://deepwiki.com/fs0414/rfmt/4-core-systems.html#nodetype-taxonomy)
* [Location Tracking](https://deepwiki.com/fs0414/rfmt/4-core-systems.html#location-tracking)
* [Comment Representation](https://deepwiki.com/fs0414/rfmt/4-core-systems.html#comment-representation)
* [Data Flow Pipeline](https://deepwiki.com/fs0414/rfmt/4-core-systems.html#data-flow-pipeline)
* [Component Overview](https://deepwiki.com/fs0414/rfmt/4-core-systems.html#component-overview)
* [4.1 AST Representation](https://deepwiki.com/fs0414/rfmt/4-core-systems.html#41-ast-representation)
* [4.2 Formatting Engine (Emitter)](https://deepwiki.com/fs0414/rfmt/4-core-systems.html#42-formatting-engine-emitter)
* [4.3 Parser Integration (PrismBridge)](https://deepwiki.com/fs0414/rfmt/4-core-systems.html#43-parser-integration-prismbridge)
* [4.4 Configuration System](https://deepwiki.com/fs0414/rfmt/4-core-systems.html#44-configuration-system)
* [4.5 Caching System](https://deepwiki.com/fs0414/rfmt/4-core-systems.html#45-caching-system)
* [4.6 Logging System](https://deepwiki.com/fs0414/rfmt/4-core-systems.html#46-logging-system)
* [Dispatch and Specialization Pattern](https://deepwiki.com/fs0414/rfmt/4-core-systems.html#dispatch-and-specialization-pattern)
* [Specialized vs Generic Formatting](https://deepwiki.com/fs0414/rfmt/4-core-systems.html#specialized-vs-generic-formatting)
* [Comment Preservation Strategy](https://deepwiki.com/fs0414/rfmt/4-core-systems.html#comment-preservation-strategy)
* [Metadata Extraction](https://deepwiki.com/fs0414/rfmt/4-core-systems.html#metadata-extraction)
* [Indentation Management](https://deepwiki.com/fs0414/rfmt/4-core-systems.html#indentation-management)
* [Block Style Detection](https://deepwiki.com/fs0414/rfmt/4-core-systems.html#block-style-detection)
* [Structural Node Filtering](https://deepwiki.com/fs0414/rfmt/4-core-systems.html#structural-node-filtering)
* [Summary](https://deepwiki.com/fs0414/rfmt/4-core-systems.html#summary)
