mod ast;
mod config;
mod emitter;
mod error;
mod logging;
mod parser;
mod policy;

use policy::SecurityPolicy;

use config::Config;
use emitter::Emitter;
use magnus::{define_module, function, prelude::*, Error, Ruby};
use parser::{PrismAdapter, RubyParser};

fn format_ruby_code(ruby: &Ruby, source: String, json: String) -> Result<String, Error> {
    log::info!("format_ruby_code called");
    let policy = SecurityPolicy::default();

    policy
        .validate_source_size(&source)
        .map_err(|e| e.to_magnus_error(ruby))?;

    log::debug!("Source code validated, size: {} bytes", source.len());

    let parser = PrismAdapter::new();
    let ast = parser.parse(&json).map_err(|e| e.to_magnus_error(ruby))?;

    // Load configuration from file or use defaults
    log::info!("Attempting to discover config file...");
    let config = Config::discover().map_err(|e| e.to_magnus_error(ruby))?;
    log::info!(
        "Config loaded successfully, line_length: {}",
        config.formatting.line_length
    );
    let mut emitter = Emitter::with_source(config, source);

    let formatted = emitter.emit(&ast).map_err(|e| e.to_magnus_error(ruby))?;

    Ok(formatted)
}

/// Parse Ruby source code and return JSON AST representation
/// This is useful for debugging and integration testing
fn parse_to_json(ruby: &Ruby, source: String) -> Result<String, Error> {
    let parser = PrismAdapter::new();
    let ast = parser.parse(&source).map_err(|e| e.to_magnus_error(ruby))?;

    Ok(format!("{:#?}", ast))
}

fn rust_version() -> String {
    "0.2.0 (Rust)".to_string()
}

#[magnus::init]
fn init(_ruby: &Ruby) -> Result<(), Error> {
    logging::RfmtLogger::init();
    log::info!("Initializing rfmt Rust extension");

    let module = define_module("Rfmt")?;

    module.define_singleton_method("format_code", function!(format_ruby_code, 2))?;
    module.define_singleton_method("parse_to_json", function!(parse_to_json, 1))?;
    module.define_singleton_method("rust_version", function!(rust_version, 0))?;

    log::info!("rfmt Rust extension initialized successfully");
    Ok(())
}
