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
use parser::{NativeAdapter, RubyParser};

fn format_ruby_code(ruby: &Ruby, source: String) -> Result<String, Error> {
    format_impl(ruby, source, None)
}

// Separate fixed-arity export instead of a variadic `format_code`: magnus
// handles fixed signatures natively, so no scan_args parsing to get wrong.
fn format_ruby_code_with_config(
    ruby: &Ruby,
    source: String,
    config_path: Option<String>,
) -> Result<String, Error> {
    format_impl(ruby, source, config_path)
}

fn format_impl(ruby: &Ruby, source: String, config_path: Option<String>) -> Result<String, Error> {
    let policy = SecurityPolicy::default();

    policy
        .validate_source_size(&source)
        .map_err(|e| e.to_magnus_error(ruby))?;

    let parser = NativeAdapter::new();
    let ast = parser.parse(&source).map_err(|e| e.to_magnus_error(ruby))?;

    let config = Config::resolve(config_path.as_deref().map(std::path::Path::new))
        .map_err(|e| e.to_magnus_error(ruby))?;
    let formatter = Formatter::new(config);

    let formatted = formatter
        .format(&source, &ast)
        .map_err(|e| e.to_magnus_error(ruby))?;

    validation::validate_output(&formatted).map_err(|e| e.to_magnus_error(ruby))?;

    Ok(formatted)
}

/// Serialize the effective configuration so Ruby can display exactly what
/// the formatter will use (CLI `config` command, --config fail-fast check)
fn resolved_config_yaml(ruby: &Ruby, config_path: Option<String>) -> Result<String, Error> {
    let config = Config::resolve(config_path.as_deref().map(std::path::Path::new))
        .map_err(|e| e.to_magnus_error(ruby))?;

    serde_yaml::to_string(&config)
        .map_err(|e| Error::new(ruby.exception_standard_error(), e.to_string()))
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
    logging::KenshinLogger::init();

    let module = ruby.define_module("Kenshin")?;

    module.define_singleton_method("format_code", function!(format_ruby_code, 1))?;
    module.define_singleton_method(
        "format_code_with_config",
        function!(format_ruby_code_with_config, 2),
    )?;
    module.define_singleton_method("parse_to_json", function!(parse_to_json, 1))?;
    module.define_singleton_method("resolved_config_yaml", function!(resolved_config_yaml, 1))?;
    module.define_singleton_method("rust_version", function!(rust_version, 0))?;

    Ok(())
}
