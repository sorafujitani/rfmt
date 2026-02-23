//! Formatting rules for different AST node types
//!
//! This module contains the FormatRule implementations for each supported
//! node type. Each rule is responsible for converting its node type to Doc IR.

mod begin;
mod body_end;
mod call;
mod case;
mod class;
mod def;
mod fallback;
mod if_unless;
mod loops;
mod module;

pub use begin::{BeginRule, EnsureRule, RescueRule};
pub use call::{BlockRule, CallRule, LambdaRule};
pub use case::{CaseMatchRule, CaseRule, InRule, WhenRule};
pub use class::ClassRule;
pub use def::DefRule;
pub use fallback::FallbackRule;
pub use if_unless::{IfRule, UnlessRule};
pub use loops::{ForRule, UntilRule, WhileRule};
pub use module::ModuleRule;
