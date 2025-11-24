use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use log::{debug, info, warn};
use crate::error::{RfmtError, Result};

/// Complete configuration structure matching .rfmt.yml format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub version: String,

    #[serde(default)]
    pub parser: ParserConfig,

    #[serde(default)]
    pub formatting: FormattingConfig,

    #[serde(default)]
    pub include: Vec<String>,

    #[serde(default)]
    pub exclude: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParserConfig {
    pub version: String,
    pub error_tolerance: bool,
    pub encoding: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormattingConfig {
    #[serde(default = "default_line_length")]
    pub line_length: usize,

    #[serde(rename = "indent_style", default)]
    pub indent_style: IndentStyle,

    #[serde(rename = "indent_width", default = "default_indent_width")]
    pub indent_width: usize,

    #[serde(rename = "quote_style", default)]
    pub quote_style: QuoteStyle,

    #[serde(default)]
    pub style: StyleConfig,
}

fn default_line_length() -> usize {
    100
}

fn default_indent_width() -> usize {
    2
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IndentStyle {
    Spaces,
    Tabs,
}

impl Default for IndentStyle {
    fn default() -> Self {
        Self::Spaces
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StyleConfig {
    #[serde(default)]
    pub quotes: QuoteStyle,

    #[serde(default)]
    pub hash_syntax: HashSyntax,

    #[serde(default)]
    pub trailing_comma: TrailingComma,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum QuoteStyle {
    Double,
    Single,
    Consistent,
}

impl Default for QuoteStyle {
    fn default() -> Self {
        Self::Double
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HashSyntax {
    Ruby19,
    HashRockets,
    Consistent,
}

impl Default for HashSyntax {
    fn default() -> Self {
        Self::Ruby19
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TrailingComma {
    Always,
    Never,
    Multiline,
}

impl Default for TrailingComma {
    fn default() -> Self {
        Self::Multiline
    }
}

impl Config {
    /// Load configuration with precedence: project → user → default
    pub fn load() -> Result<Self> {
        Self::load_from_path(None)
    }

    /// Load configuration from specific path or use precedence
    pub fn load_from_path(explicit_path: Option<&Path>) -> Result<Self> {
        if let Some(path) = explicit_path {
            info!("Loading configuration from explicit path: {:?}", path);
            return Self::load_file(path);
        }

        // Precedence: current directory → parent directories → user home → default
        let config_paths = Self::find_config_files();

        for path in &config_paths {
            debug!("Checking config file: {:?}", path);
            if path.exists() {
                info!("Found configuration file: {:?}", path);
                return Self::load_file(path);
            }
        }

        warn!("No configuration file found, using defaults");
        Ok(Self::default())
    }

    /// Find all potential configuration file paths in order of precedence
    fn find_config_files() -> Vec<PathBuf> {
        let mut paths = Vec::new();

        // 1. Current directory and parents (walk up the tree)
        if let Ok(current_dir) = std::env::current_dir() {
            let mut dir = current_dir.as_path();
            loop {
                paths.push(dir.join(".rfmt.yml"));
                paths.push(dir.join(".rfmt.yaml"));

                match dir.parent() {
                    Some(parent) => dir = parent,
                    None => break,
                }
            }
        }

        // 2. User home directory
        if let Some(home_dir) = dirs::home_dir() {
            paths.push(home_dir.join(".rfmt.yml"));
            paths.push(home_dir.join(".rfmt.yaml"));
        }

        paths
    }

    /// Load configuration from a specific file
    fn load_file(path: &Path) -> Result<Self> {
        let contents = fs::read_to_string(path).map_err(|e| RfmtError::IoError {
            file: path.to_path_buf(),
            message: "Failed to read configuration file".to_string(),
            source: e,
        })?;

        let config: Config = serde_yaml::from_str(&contents).map_err(|e| {
            RfmtError::ConfigError {
                message: format!("Failed to parse YAML: {}", e),
                file: path.to_path_buf(),
                suggestion: "Check YAML syntax and ensure all keys are valid".to_string(),
            }
        })?;

        config.validate(path)?;

        info!("Configuration loaded successfully from {:?}", path);
        Ok(config)
    }

    /// Validate configuration values
    fn validate(&self, path: &Path) -> Result<()> {
        // Validate line length
        if self.formatting.line_length < 40 || self.formatting.line_length > 500 {
            return Err(RfmtError::ConfigError {
                message: format!(
                    "Invalid line_length: {}. Must be between 40 and 500",
                    self.formatting.line_length
                ),
                file: path.to_path_buf(),
                suggestion: "Use a value between 40 and 500, commonly 80, 100, or 120".to_string(),
            });
        }

        // Validate indent width
        if self.formatting.indent_width < 1 || self.formatting.indent_width > 8 {
            return Err(RfmtError::ConfigError {
                message: format!(
                    "Invalid indent_width: {}. Must be between 1 and 8",
                    self.formatting.indent_width
                ),
                file: path.to_path_buf(),
                suggestion: "Use 2 for Ruby (standard), or 4 for other preferences".to_string(),
            });
        }

        // Validate version format if specified
        if !self.version.is_empty() && self.version != "1.0" {
            warn!(
                "Configuration version '{}' may not be supported. Current version: 1.0",
                self.version
            );
        }

        // Validate glob patterns
        for pattern in &self.include {
            if pattern.is_empty() {
                return Err(RfmtError::ConfigError {
                    message: "Empty pattern in 'include' list".to_string(),
                    file: path.to_path_buf(),
                    suggestion: "Remove empty patterns or use valid glob patterns".to_string(),
                });
            }
        }

        for pattern in &self.exclude {
            if pattern.is_empty() {
                return Err(RfmtError::ConfigError {
                    message: "Empty pattern in 'exclude' list".to_string(),
                    file: path.to_path_buf(),
                    suggestion: "Remove empty patterns or use valid glob patterns".to_string(),
                });
            }
        }

        debug!("Configuration validation passed");
        Ok(())
    }

    /// Get indent string based on configuration
    pub fn indent_string(&self) -> String {
        match self.formatting.indent_style {
            IndentStyle::Spaces => " ".repeat(self.formatting.indent_width),
            IndentStyle::Tabs => "\t".to_string(),
        }
    }

    /// Check if a file path should be included based on patterns
    pub fn should_include(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();

        // Check exclude patterns first (takes precedence)
        for pattern in &self.exclude {
            if Self::matches_glob(&path_str, pattern) {
                debug!("File {:?} excluded by pattern: {}", path, pattern);
                return false;
            }
        }

        // If no include patterns specified, include everything not excluded
        if self.include.is_empty() {
            return true;
        }

        // Check include patterns
        for pattern in &self.include {
            if Self::matches_glob(&path_str, pattern) {
                debug!("File {:?} included by pattern: {}", path, pattern);
                return true;
            }
        }

        false
    }

    /// Simple glob pattern matching
    fn matches_glob(path: &str, pattern: &str) -> bool {
        // Simple implementation - in production, use a proper glob library
        if pattern.contains("**") {
            // Match any directory depth
            // Example: "**/*.rb" matches "lib/foo.rb", "vendor/gems/foo.rb"
            // Example: "vendor/**/*" matches "vendor/gems/foo.rb"
            let parts: Vec<&str> = pattern.split("**").collect();
            if parts.len() == 2 {
                let prefix = parts[0];
                let suffix = parts[1].trim_start_matches('/');

                // Check prefix match (if prefix exists)
                if !prefix.is_empty() && !path.starts_with(prefix) {
                    return false;
                }

                // Handle suffix matching
                if suffix.is_empty() {
                    // Pattern like "vendor/**" - matches everything under vendor
                    return prefix.is_empty() || path.starts_with(prefix);
                } else if suffix.starts_with('*') {
                    // Pattern like "**/*.rb" - check if path ends with the extension
                    let ext = suffix.trim_start_matches('*');
                    return path.ends_with(ext);
                } else {
                    // Other patterns - exact suffix match
                    return path.ends_with(suffix);
                }
            }
        } else if pattern.contains('*') {
            // Simple wildcard matching
            // Example: "*.rb" matches "foo.rb"
            let parts: Vec<&str> = pattern.split('*').collect();
            if parts.len() == 2 {
                let prefix = parts[0];
                let suffix = parts[1];

                if prefix.is_empty() {
                    // Pattern like "*.rb"
                    return path.ends_with(suffix);
                } else if suffix.is_empty() {
                    // Pattern like "foo*"
                    return path.starts_with(prefix);
                } else {
                    // Pattern like "foo*.rb"
                    return path.starts_with(prefix) && path.ends_with(suffix);
                }
            }
        } else {
            // Exact match
            return path == pattern;
        }
        false
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            version: "1.0".to_string(),
            parser: ParserConfig::default(),
            formatting: FormattingConfig::default(),
            include: vec![
                "**/*.rb".to_string(),
                "**/*.rake".to_string(),
            ],
            exclude: vec![
                "vendor/**/*".to_string(),
                "tmp/**/*".to_string(),
                "node_modules/**/*".to_string(),
            ],
        }
    }
}

impl Default for ParserConfig {
    fn default() -> Self {
        Self {
            version: "latest".to_string(),
            error_tolerance: true,
            encoding: "UTF-8".to_string(),
        }
    }
}

impl Default for FormattingConfig {
    fn default() -> Self {
        Self {
            line_length: 100,
            indent_style: IndentStyle::Spaces,
            indent_width: 2,
            quote_style: QuoteStyle::Double,
            style: StyleConfig::default(),
        }
    }
}

impl Default for StyleConfig {
    fn default() -> Self {
        Self {
            quotes: QuoteStyle::Double,
            hash_syntax: HashSyntax::Ruby19,
            trailing_comma: TrailingComma::Multiline,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.version, "1.0");
        assert_eq!(config.formatting.line_length, 100);
        assert_eq!(config.formatting.indent_width, 2);
        assert!(matches!(config.formatting.indent_style, IndentStyle::Spaces));
    }

    #[test]
    fn test_load_valid_config() {
        let yaml = r#"
version: "1.0"
formatting:
  line_length: 120
  indent_width: 4
  indent_style: tabs
  quote_style: single
include:
  - "**/*.rb"
exclude:
  - "vendor/**/*"
"#;

        let mut file = NamedTempFile::new().unwrap();
        file.write_all(yaml.as_bytes()).unwrap();
        file.flush().unwrap();

        let config = Config::load_file(file.path()).unwrap();
        assert_eq!(config.formatting.line_length, 120);
        assert_eq!(config.formatting.indent_width, 4);
        assert!(matches!(config.formatting.indent_style, IndentStyle::Tabs));
        assert!(matches!(config.formatting.quote_style, QuoteStyle::Single));
    }

    #[test]
    fn test_validate_line_length_too_small() {
        let yaml = r#"
formatting:
  line_length: 30
"#;

        let mut file = NamedTempFile::new().unwrap();
        file.write_all(yaml.as_bytes()).unwrap();
        file.flush().unwrap();

        let result = Config::load_file(file.path());
        assert!(result.is_err());
        if let Err(RfmtError::ConfigError { message, .. }) = result {
            assert!(message.contains("line_length"));
            assert!(message.contains("40 and 500"));
        }
    }

    #[test]
    fn test_validate_line_length_too_large() {
        let yaml = r#"
formatting:
  line_length: 600
"#;

        let mut file = NamedTempFile::new().unwrap();
        file.write_all(yaml.as_bytes()).unwrap();
        file.flush().unwrap();

        let result = Config::load_file(file.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_indent_width() {
        let yaml = r#"
formatting:
  indent_width: 10
"#;

        let mut file = NamedTempFile::new().unwrap();
        file.write_all(yaml.as_bytes()).unwrap();
        file.flush().unwrap();

        let result = Config::load_file(file.path());
        assert!(result.is_err());
        if let Err(RfmtError::ConfigError { message, .. }) = result {
            assert!(message.contains("indent_width"));
        }
    }

    #[test]
    fn test_indent_string_spaces() {
        let config = Config::default();
        assert_eq!(config.indent_string(), "  "); // 2 spaces
    }

    #[test]
    fn test_indent_string_tabs() {
        let mut config = Config::default();
        config.formatting.indent_style = IndentStyle::Tabs;
        assert_eq!(config.indent_string(), "\t");
    }

    #[test]
    fn test_should_include_basic() {
        let config = Config::default();
        assert!(config.should_include(Path::new("lib/foo.rb")));
        assert!(!config.should_include(Path::new("vendor/gem/foo.rb")));
    }

    #[test]
    fn test_should_include_with_exclude() {
        let mut config = Config::default();
        config.exclude.push("test/**/*".to_string());
        assert!(!config.should_include(Path::new("test/foo.rb")));
    }

    #[test]
    fn test_invalid_yaml_syntax() {
        let yaml = r#"
formatting:
  line_length: not_a_number
"#;

        let mut file = NamedTempFile::new().unwrap();
        file.write_all(yaml.as_bytes()).unwrap();
        file.flush().unwrap();

        let result = Config::load_file(file.path());
        assert!(result.is_err());
        if let Err(RfmtError::ConfigError { message, .. }) = result {
            assert!(message.contains("parse"));
        }
    }

    #[test]
    fn test_partial_config_uses_defaults() {
        let yaml = r#"
formatting:
  line_length: 80
"#;

        let mut file = NamedTempFile::new().unwrap();
        file.write_all(yaml.as_bytes()).unwrap();
        file.flush().unwrap();

        let config = Config::load_file(file.path()).unwrap();
        assert_eq!(config.formatting.line_length, 80);
        assert_eq!(config.formatting.indent_width, 2); // default
        assert!(matches!(config.formatting.indent_style, IndentStyle::Spaces)); // default
    }
}
