pub mod ast;
pub mod config;
pub mod doc;
pub mod error;
pub mod format;
mod logging;
pub mod parser;
mod policy;
pub mod validation;

use policy::SecurityPolicy;

use config::Config;
use format::Formatter;
use magnus::{function, prelude::*, Error, Ruby};
use parser::{NativeAdapter, PrismAdapter, RubyParser};

fn format_ruby_code(ruby: &Ruby, source: String) -> Result<String, Error> {
    let policy = SecurityPolicy::default();

    policy
        .validate_source_size(&source)
        .map_err(|e| e.to_magnus_error(ruby))?;

    let parser = NativeAdapter::new();
    let ast = parser.parse(&source).map_err(|e| e.to_magnus_error(ruby))?;

    let config = Config::discover().map_err(|e| e.to_magnus_error(ruby))?;
    let formatter = Formatter::new(config);

    let formatted = formatter
        .format(&source, &ast)
        .map_err(|e| e.to_magnus_error(ruby))?;

    validation::validate_output(&formatted).map_err(|e| e.to_magnus_error(ruby))?;

    Ok(formatted)
}

// Temporary for the prism migration: the pre-switchover pipeline (Ruby Prism
// JSON in, no output validation), kept so differential_check.rb can compare
// it against the native path. Deleted in phase 7.
fn format_ruby_code_legacy(ruby: &Ruby, source: String, json: String) -> Result<String, Error> {
    let policy = SecurityPolicy::default();

    policy
        .validate_source_size(&source)
        .map_err(|e| e.to_magnus_error(ruby))?;

    let parser = PrismAdapter::new();
    let ast = parser.parse(&json).map_err(|e| e.to_magnus_error(ruby))?;

    let config = Config::discover().map_err(|e| e.to_magnus_error(ruby))?;
    let formatter = Formatter::new(config);

    let formatted = formatter
        .format(&source, &ast)
        .map_err(|e| e.to_magnus_error(ruby))?;

    Ok(formatted)
}

/// Parse Ruby source code and return the internal AST representation
/// This is useful for debugging and integration testing
fn parse_to_json(ruby: &Ruby, source: String) -> Result<String, Error> {
    let parser = NativeAdapter::new();
    let ast = parser.parse(&source).map_err(|e| e.to_magnus_error(ruby))?;

    Ok(format!("{:#?}", ast))
}

fn rust_version() -> String {
    "0.2.0 (Rust)".to_string()
}

#[magnus::init]
fn init(ruby: &Ruby) -> Result<(), Error> {
    logging::RfmtLogger::init();

    let module = ruby.define_module("Rfmt")?;

    module.define_singleton_method("format_code", function!(format_ruby_code, 1))?;
    module.define_singleton_method("format_code_legacy", function!(format_ruby_code_legacy, 2))?;
    module.define_singleton_method("parse_to_json", function!(parse_to_json, 1))?;
    module.define_singleton_method("rust_version", function!(rust_version, 0))?;

    Ok(())
}
