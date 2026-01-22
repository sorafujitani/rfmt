---
title: "Formatting Behavior | fs0414/rfmt | DeepWiki"
source_url: "https://deepwiki.com/fs0414/rfmt/5-formatting-behavior"
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

# Formatting Behavior

Relevant source files

* [spec/case\_formatting\_spec.rb](https://github.com/fs0414/rfmt/blob/90c575e6/spec/case_formatting_spec.rb)
* [spec/conditional\_formatting\_spec.rb](https://github.com/fs0414/rfmt/blob/90c575e6/spec/conditional_formatting_spec.rb)
* [spec/ensure\_formatting\_spec.rb](https://github.com/fs0414/rfmt/blob/90c575e6/spec/ensure_formatting_spec.rb)
* [spec/lambda\_formatting\_spec.rb](https://github.com/fs0414/rfmt/blob/90c575e6/spec/lambda_formatting_spec.rb)
* [spec/loop\_formatting\_spec.rb](https://github.com/fs0414/rfmt/blob/90c575e6/spec/loop_formatting_spec.rb)

## Purpose and Scope

This document describes how `rfmt` transforms Ruby source code into formatted output. It covers the general principles of the formatting engine, the dispatch mechanism that routes different language constructs to specialized handlers, and examples of formatting behavior across major Ruby syntax categories.

For detailed formatting rules for specific constructs, see:

* Control flow constructs (if/unless/case/while): [Control Flow Formatting](https://deepwiki.com/fs0414/rfmt/5.1-control-flow-formatting.html)
* Exception handling (begin/rescue/ensure): [Exception Handling Formatting](https://deepwiki.com/fs0414/rfmt/5.2-exception-handling-formatting.html)
* Lambdas and blocks: [Lambda and Block Formatting](https://deepwiki.com/fs0414/rfmt/5.3-lambda-and-block-formatting.html)

This page focuses on the **how** of formatting—the mechanisms and principles. For **what** can be configured, see [Configuration Options Reference](https://deepwiki.com/fs0414/rfmt/7.2-configuration-options-reference.html).

## Formatting Pipeline Overview

The formatting process in `rfmt` follows a three-stage pipeline within the Rust `Emitter`:

**Sources:** [ext/rfmt/src/emitter/mod.rs1-500](https://github.com/fs0414/rfmt/blob/90c575e6/ext/rfmt/src/emitter/mod.rs#L1-L500) (referenced in high-level diagrams)

The `Emitter` maintains state during formatting:

| State Component | Purpose | Example |
| --- | --- | --- |
| `indent_level` | Current indentation depth | `0` at root, `1` inside class, `2` inside method |
| `config` | Formatting configuration | `line_length: 100`, `indent_width: 2` |
| `comments` | Collected comment nodes | Line comments, block comments with positions |
| `source` | Original source text | Used by `emit_generic()` for fallback |
| `output` | Accumulated formatted string | Built incrementally during traversal |

## Node Dispatch Mechanism

The `Emitter` uses pattern matching on `NodeType` to route each AST node to its appropriate handler. This dispatch mechanism is the core of `rfmt`'s formatting architecture:

**Sources:** [ext/rfmt/src/emitter/mod.rs50-200](https://github.com/fs0414/rfmt/blob/90c575e6/ext/rfmt/src/emitter/mod.rs#L50-L200) (dispatch logic), [ext/rfmt/src/ast/mod.rs1-100](https://github.com/fs0414/rfmt/blob/90c575e6/ext/rfmt/src/ast/mod.rs#L1-L100) (NodeType enum)

### Dispatch Categories

The dispatch mechanism groups node types into semantic categories:

| Category | Node Types | Handler Methods |
| --- | --- | --- |
| **Definitions** | `ClassNode`, `ModuleNode`, `DefNode`, `SingletonClassNode` | `emit_class()`, `emit_module()`, `emit_method()` |
| **Control Flow** | `IfNode`, `UnlessNode`, `CaseNode`, `WhenNode`, `WhileNode`, `UntilNode`, `ForNode` | `emit_if_unless()`, `emit_case()`, `emit_while_until()`, `emit_for()` |
| **Exception Handling** | `BeginNode`, `RescueNode`, `EnsureNode`, `RescueModifierNode` | `emit_begin()`, `emit_rescue()`, `emit_ensure()` |
| **Expressions** | `CallNode`, `LambdaNode`, `BlockNode`, `ArrayNode`, `HashNode` | `emit_call()`, `emit_lambda()`, `emit_block()` |
| **Literals** | `StringNode`, `IntegerNode`, `FloatNode`, `SymbolNode`, `InterpolatedStringNode` | `emit_string()`, `emit_literal()` |
| **Variables** | `LocalVariableReadNode`, `InstanceVariableReadNode`, `ClassVariableReadNode`, `GlobalVariableReadNode` | `emit_variable()` |

## General Formatting Principles

### Indentation Management

`rfmt` maintains consistent indentation using a stack-based approach:

**Sources:** [ext/rfmt/src/emitter/mod.rs300-400](https://github.com/fs0414/rfmt/blob/90c575e6/ext/rfmt/src/emitter/mod.rs#L300-L400) (indentation logic)

Indentation follows these rules:

1. **Base indentation**: Each new line starts with `indent_width * indent_level` spaces (or tabs if `indent_style: tabs`)
2. **Structural blocks**: Class, module, method, and control flow bodies increase indentation by 1 level
3. **Continuation lines**: Multi-line expressions maintain the indentation of their opening line
4. **Block bodies**: `do...end` and `{...}` blocks follow the same indentation rules as structural blocks

Example from [spec/conditional\_formatting\_spec.rb58-77](https://github.com/fs0414/rfmt/blob/90c575e6/spec/conditional_formatting_spec.rb#L58-L77):

### Line Break Handling

`rfmt` determines line breaks based on:

1. **Structural keywords**: Keywords like `if`, `else`, `elsif`, `case`, `when`, `rescue`, `ensure`, `end` always appear on their own lines
2. **Statement separation**: Each statement is placed on a new line unless it's a postfix conditional
3. **Block boundaries**: Opening and closing delimiters (`do`/`end`, `{`/`}`) respect single-line vs multi-line block conventions

Example of postfix conditional preservation from [spec/conditional\_formatting\_spec.rb52-56](https://github.com/fs0414/rfmt/blob/90c575e6/spec/conditional_formatting_spec.rb#L52-L56):

### Structure Preservation vs Reconstruction

`rfmt` uses two strategies for formatting:

| Strategy | When Used | Method | Example |
| --- | --- | --- | --- |
| **Reconstruction** | Simple, well-understood constructs | Custom `emit_*()` methods | `if...else...end`, `class...end` |
| **Source Extraction** | Complex constructs with intricate syntax | `emit_generic()` extracts original source | Method parameters with defaults, complex pattern matching |

The `emit_generic()` method falls back to extracting the original source text when reconstructing syntax would be error-prone:

**Sources:** [ext/rfmt/src/emitter/mod.rs450-500](https://github.com/fs0414/rfmt/blob/90c575e6/ext/rfmt/src/emitter/mod.rs#L450-L500) (emit\_generic implementation)

### Comment Preservation

Comments are preserved through a three-phase process:

1. **Collection**: `collect_comments()` extracts all comments from the AST and associates them with their positions
2. **Formatting**: Node formatting proceeds without comments, building the structural output
3. **Reinsertion**: `reinsert_comments()` places comments back at their original relative positions

This ensures comments maintain their semantic context even as code structure is reformatted.

**Sources:** [ext/rfmt/src/emitter/mod.rs100-150](https://github.com/fs0414/rfmt/blob/90c575e6/ext/rfmt/src/emitter/mod.rs#L100-L150) (comment handling)

## Formatting Examples by Category

### Control Flow Constructs

**Conditional statements** with proper indentation from [spec/conditional\_formatting\_spec.rb6-29](https://github.com/fs0414/rfmt/blob/90c575e6/spec/conditional_formatting_spec.rb#L6-L29):

**Case statements** from [spec/case\_formatting\_spec.rb6-13](https://github.com/fs0414/rfmt/blob/90c575e6/spec/case_formatting_spec.rb#L6-L13):

**Loop constructs** from [spec/loop\_formatting\_spec.rb6-12](https://github.com/fs0414/rfmt/blob/90c575e6/spec/loop_formatting_spec.rb#L6-L12) [spec/loop\_formatting\_spec.rb14-19](https://github.com/fs0414/rfmt/blob/90c575e6/spec/loop_formatting_spec.rb#L14-L19) [spec/loop\_formatting\_spec.rb21-26](https://github.com/fs0414/rfmt/blob/90c575e6/spec/loop_formatting_spec.rb#L21-L26):

**Flow control keywords** in loops from [spec/loop\_formatting\_spec.rb28-33](https://github.com/fs0414/rfmt/blob/90c575e6/spec/loop_formatting_spec.rb#L28-L33):

For comprehensive control flow formatting rules, see [Control Flow Formatting](https://deepwiki.com/fs0414/rfmt/5.1-control-flow-formatting.html).

**Sources:** [spec/conditional\_formatting\_spec.rb1-107](https://github.com/fs0414/rfmt/blob/90c575e6/spec/conditional_formatting_spec.rb#L1-L107) [spec/case\_formatting\_spec.rb1-23](https://github.com/fs0414/rfmt/blob/90c575e6/spec/case_formatting_spec.rb#L1-L23) [spec/loop\_formatting\_spec.rb1-35](https://github.com/fs0414/rfmt/blob/90c575e6/spec/loop_formatting_spec.rb#L1-L35)

### Exception Handling Constructs

**Begin-rescue-ensure blocks** from [spec/ensure\_formatting\_spec.rb6-12](https://github.com/fs0414/rfmt/blob/90c575e6/spec/ensure_formatting_spec.rb#L6-L12):

**Methods with implicit ensure** from [spec/ensure\_formatting\_spec.rb14-20](https://github.com/fs0414/rfmt/blob/90c575e6/spec/ensure_formatting_spec.rb#L14-L20):

For detailed exception handling formatting, see [Exception Handling Formatting](https://deepwiki.com/fs0414/rfmt/5.2-exception-handling-formatting.html).

**Sources:** [spec/ensure\_formatting\_spec.rb1-22](https://github.com/fs0414/rfmt/blob/90c575e6/spec/ensure_formatting_spec.rb#L1-L22)

### Lambda and Block Constructs

**Stabby lambda syntax** from [spec/lambda\_formatting\_spec.rb6-10](https://github.com/fs0414/rfmt/blob/90c575e6/spec/lambda_formatting_spec.rb#L6-L10):

**Rails scope definitions with lambdas** from [spec/lambda\_formatting\_spec.rb12-16](https://github.com/fs0414/rfmt/blob/90c575e6/spec/lambda_formatting_spec.rb#L12-L16):

For comprehensive lambda and block formatting rules, see [Lambda and Block Formatting](https://deepwiki.com/fs0414/rfmt/5.3-lambda-and-block-formatting.html).

**Sources:** [spec/lambda\_formatting\_spec.rb1-18](https://github.com/fs0414/rfmt/blob/90c575e6/spec/lambda_formatting_spec.rb#L1-L18)

### Nested Context Formatting

`rfmt` properly handles nested constructs by stacking indentation levels. Example from [spec/conditional\_formatting\_spec.rb79-106](https://github.com/fs0414/rfmt/blob/90c575e6/spec/conditional_formatting_spec.rb#L79-L106):

The indentation stack during this formatting:

| Position | Context | Indent Level |
| --- | --- | --- |
| `class Validator` | Top level | 0 |
| `def check(value)` | Inside class | 1 |
| `if value > 0` | Inside method | 2 |
| `:positive` | Inside if block | 3 |

**Sources:** [spec/conditional\_formatting\_spec.rb79-106](https://github.com/fs0414/rfmt/blob/90c575e6/spec/conditional_formatting_spec.rb#L79-L106)

## Formatting Configuration Impact

While this page focuses on **how** formatting works, it's important to note that several configuration options affect the output:

| Configuration Option | Impact on Formatting | Default |
| --- | --- | --- |
| `indent_width` | Number of spaces per indentation level | `2` |
| `indent_style` | Use spaces or tabs | `spaces` |
| `line_length` | Maximum line length (affects line breaks for long expressions) | `100` |
| `quote_style` | String literal quote style (`single`, `double`) | `double` |
| `hash_syntax` | Hash key syntax (`ruby19`, `hash_rockets`) | `ruby19` |
| `trailing_comma` | Trailing comma rules (`never`, `multiline`, `always`) | `multiline` |

These options are loaded from `.rfmt.yml` and passed to the Rust `Emitter` via the FFI boundary. For complete configuration details, see [Configuration](https://deepwiki.com/fs0414/rfmt/2.2-configuration.html) and [Configuration Options Reference](https://deepwiki.com/fs0414/rfmt/7.2-configuration-options-reference.html).

**Sources:** [lib/rfmt/cli.rb1-200](https://github.com/fs0414/rfmt/blob/90c575e6/lib/rfmt/cli.rb#L1-L200) (configuration loading), [ext/rfmt/src/emitter/mod.rs1-50](https://github.com/fs0414/rfmt/blob/90c575e6/ext/rfmt/src/emitter/mod.rs#L1-L50) (configuration application)

---

The formatting behavior described in this document is tested extensively through RSpec test suites. Each test validates that `rfmt` produces correctly formatted output for specific Ruby constructs. The test suite ensures formatting is consistent, preserves semantic meaning, and maintains comment positions across all supported syntax.

Dismiss

Refresh this wiki

Enter email to refresh

### On this page

* [Formatting Behavior](https://deepwiki.com/fs0414/rfmt/5-formatting-behavior.html#formatting-behavior)
* [Purpose and Scope](https://deepwiki.com/fs0414/rfmt/5-formatting-behavior.html#purpose-and-scope)
* [Formatting Pipeline Overview](https://deepwiki.com/fs0414/rfmt/5-formatting-behavior.html#formatting-pipeline-overview)
* [Node Dispatch Mechanism](https://deepwiki.com/fs0414/rfmt/5-formatting-behavior.html#node-dispatch-mechanism)
* [Dispatch Categories](https://deepwiki.com/fs0414/rfmt/5-formatting-behavior.html#dispatch-categories)
* [General Formatting Principles](https://deepwiki.com/fs0414/rfmt/5-formatting-behavior.html#general-formatting-principles)
* [Indentation Management](https://deepwiki.com/fs0414/rfmt/5-formatting-behavior.html#indentation-management)
* [Line Break Handling](https://deepwiki.com/fs0414/rfmt/5-formatting-behavior.html#line-break-handling)
* [Structure Preservation vs Reconstruction](https://deepwiki.com/fs0414/rfmt/5-formatting-behavior.html#structure-preservation-vs-reconstruction)
* [Comment Preservation](https://deepwiki.com/fs0414/rfmt/5-formatting-behavior.html#comment-preservation)
* [Formatting Examples by Category](https://deepwiki.com/fs0414/rfmt/5-formatting-behavior.html#formatting-examples-by-category)
* [Control Flow Constructs](https://deepwiki.com/fs0414/rfmt/5-formatting-behavior.html#control-flow-constructs)
* [Exception Handling Constructs](https://deepwiki.com/fs0414/rfmt/5-formatting-behavior.html#exception-handling-constructs)
* [Lambda and Block Constructs](https://deepwiki.com/fs0414/rfmt/5-formatting-behavior.html#lambda-and-block-constructs)
* [Nested Context Formatting](https://deepwiki.com/fs0414/rfmt/5-formatting-behavior.html#nested-context-formatting)
* [Formatting Configuration Impact](https://deepwiki.com/fs0414/rfmt/5-formatting-behavior.html#formatting-configuration-impact)
