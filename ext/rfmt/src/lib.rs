// Core modules
mod ast;
mod parser;
mod config;
mod formatter;
mod emitter;
mod error;

// Optional modules
mod cache;
mod plugin;

// Phase 4: Logging and debugging modules
mod logging;
mod debug;

// Export debug macros
#[allow(unused_imports)]
use debug::*;

use magnus::{define_module, function, prelude::*, Error, Ruby};
use parser::{PrismAdapter, RubyParser};
use formatter::Formatter;
use emitter::Emitter;
use config::Config;

fn format_ruby_code(ruby: &Ruby, source: String, json: String) -> Result<String, Error> {
    // Parse JSON to internal AST
    let parser = PrismAdapter::new();
    let ast = parser.parse(&json)
        .map_err(|e| e.to_magnus_error(ruby))?;

    // Create emitter with source code for fallback extraction
    let config = Config::default();
    let mut emitter = Emitter::with_source(config, source);

    // Emit formatted code
    let formatted = emitter.emit(&ast)
        .map_err(|e| e.to_magnus_error(ruby))?;

    Ok(formatted)
}

/// Parse Ruby source code and return JSON AST representation
/// This is useful for debugging and integration testing
fn parse_to_json(ruby: &Ruby, source: String) -> Result<String, Error> {
    // This function expects JSON input from Ruby's PrismBridge
    // and returns the parsed AST as a debug string
    let parser = PrismAdapter::new();
    let ast = parser.parse(&source)
        .map_err(|e| e.to_magnus_error(ruby))?;

    // For now, return debug representation
    // In Phase 2, we might want to serialize back to JSON
    Ok(format!("{:#?}", ast))
}

fn rust_version() -> String {
    "0.1.0 (Rust)".to_string()
}

#[magnus::init]
fn init(ruby: &Ruby) -> Result<(), Error> {
    // Initialize logging system
    logging::RfmtLogger::init();
    log::info!("Initializing rfmt Rust extension");

    let module = define_module("Rfmt")?;

    // Main formatting function
    module.define_singleton_method("format_code", function!(format_ruby_code, 2))?;

    // Debug/testing function
    module.define_singleton_method("parse_to_json", function!(parse_to_json, 1))?;

    // Version info
    module.define_singleton_method("rust_version", function!(rust_version, 0))?;

    // Define custom exception classes for better error handling
    let rfmt_error = ruby.define_error("RfmtError", ruby.exception_standard_error())?;
    ruby.define_error("ParseError", rfmt_error)?;
    ruby.define_error("ConfigError", rfmt_error)?;
    ruby.define_error("PrismError", rfmt_error)?;
    ruby.define_error("RuleError", rfmt_error)?;
    ruby.define_error("InternalError", rfmt_error)?;
    ruby.define_error("FormattingError", rfmt_error)?;
    ruby.define_error("UnsupportedFeature", rfmt_error)?;

    log::info!("rfmt Rust extension initialized successfully");
    Ok(())
}