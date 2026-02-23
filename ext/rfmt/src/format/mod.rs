//! Rule-based formatting system for rfmt
//!
//! This module provides a Prettier-inspired architecture for formatting Ruby code:
//!
//! - **FormatContext**: Manages state during formatting (comments, source, config)
//! - **FormatRule**: Trait for implementing formatting rules for specific node types
//! - **RuleRegistry**: Maps node types to their formatting rules
//! - **Formatter**: Main entry point that coordinates the formatting process
//!
//! # Architecture
//!
//! ```text
//! AST Node → RuleRegistry.get_rule() → FormatRule.format() → Doc IR → Printer → String
//! ```
//!
//! # Example
//!
//! ```rust,ignore
//! use rfmt::format::Formatter;
//! use rfmt::config::Config;
//!
//! let formatter = Formatter::new(Config::default());
//! let result = formatter.format(source, &ast)?;
//! ```

pub mod context;
pub mod formatter;
pub mod registry;
pub mod rule;
pub mod rules;

pub use context::FormatContext;
pub use formatter::Formatter;
pub use registry::RuleRegistry;
pub use rule::{BoxedRule, FormatRule};
