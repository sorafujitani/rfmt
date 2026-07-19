use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::SystemTime;

/// Complete configuration structure matching .kenshin.yml format
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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IndentStyle {
    #[default]
    Spaces,
    Tabs,
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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum QuoteStyle {
    #[default]
    Double,
    Single,
    Consistent,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub enum HashSyntax {
    #[default]
    Ruby19,
    HashRockets,
    Consistent,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TrailingComma {
    Always,
    Never,
    #[default]
    Multiline,
}

/// Search order within each directory: kenshin.yml, kenshin.yaml, .kenshin.yml, .kenshin.yaml.
/// The rfmt names are accepted after them during the rename transition window
/// (planned removal one minor release after 1.7).
const CONFIG_FILE_NAMES: [&str; 8] = [
    "kenshin.yml",
    "kenshin.yaml",
    ".kenshin.yml",
    ".kenshin.yaml",
    "rfmt.yml",
    "rfmt.yaml",
    ".rfmt.yml",
    ".rfmt.yaml",
];

/// Discovery result cached per process so repeated format calls (CLI batch,
/// long-lived LSP) skip the cwd-to-root-to-home filesystem walk.
struct DiscoveryCache {
    cwd: PathBuf,
    /// Discovered file and its mtime; `None` when nothing was found.
    found: Option<(PathBuf, SystemTime)>,
    config: Config,
}

static DISCOVERY_CACHE: Mutex<Option<DiscoveryCache>> = Mutex::new(None);

/// Explicit-path loads cached separately, keyed by canonical path + mtime,
/// so a CLI batch does not re-parse the same YAML once per file.
static EXPLICIT_CACHE: Mutex<Option<(PathBuf, SystemTime, Config)>> = Mutex::new(None);

impl Config {
    /// Resolve the effective configuration.
    ///
    /// Error handling differs by intent: an explicit path was asked for by
    /// name, so load failures must surface loudly; discovery merely stumbles
    /// on files, so a broken discovered file logs a warning and falls back
    /// to defaults (an LSP mid-edit of .kenshin.yml must not break formatting).
    pub fn resolve(explicit_path: Option<&Path>) -> crate::error::Result<Self> {
        match explicit_path {
            Some(path) => Self::load_explicit_cached(path),
            None => {
                let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
                Ok(Self::discover_cached_from(&cwd))
            }
        }
    }

    fn load_explicit_cached(path: &Path) -> crate::error::Result<Self> {
        // Canonicalize so a relative path is not confused across cwd changes.
        let key = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
        // mtime read before the load: a racing write can only make the cache
        // entry look older than the content, forcing a reload, never staleness.
        let mtime = file_mtime(&key);

        let mut guard = EXPLICIT_CACHE
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        if let (Some((cached_path, cached_mtime, config)), Some(mtime)) = (guard.as_ref(), mtime) {
            if *cached_path == key && *cached_mtime == mtime {
                return Ok(config.clone());
            }
        }

        let config = Self::load_file(path)?;
        if let Some(mtime) = mtime {
            *guard = Some((key, mtime, config.clone()));
        }
        Ok(config)
    }

    fn discover_cached_from(cwd: &Path) -> Self {
        let mut guard = DISCOVERY_CACHE
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        if let Some(cache) = guard.as_ref() {
            if cache.cwd == cwd && cache_is_fresh(cwd, cache) {
                return cache.config.clone();
            }
        }

        let (config, found) = Self::discover_from(cwd);
        *guard = Some(DiscoveryCache {
            cwd: cwd.to_path_buf(),
            found,
            config: config.clone(),
        });
        config
    }

    /// Walk `start` up to the root, then the home directory.
    fn find_config_file(start: &Path) -> Option<PathBuf> {
        let mut current_dir = start.to_path_buf();
        loop {
            if let Some(path) = first_candidate_in(&current_dir) {
                return Some(path);
            }
            if !current_dir.pop() {
                break;
            }
        }

        dirs::home_dir().and_then(|home| first_candidate_in(&home))
    }

    fn discover_from(start: &Path) -> (Self, Option<(PathBuf, SystemTime)>) {
        let Some(path) = Self::find_config_file(start) else {
            log::info!("No config file found, using defaults");
            return (Config::default(), None);
        };

        // A broken file is still cached with its mtime so fixing it triggers a reload.
        let config = Self::load_file(&path).unwrap_or_else(|e| {
            log::warn!("Ignoring config file {:?}: {}", path, e);
            Config::default()
        });
        let mtime = file_mtime(&path).unwrap_or(SystemTime::UNIX_EPOCH);
        log::info!("Found config file: {:?}", path);
        (config, Some((path, mtime)))
    }

    /// Load configuration from a YAML file
    pub fn load_file(path: &std::path::Path) -> crate::error::Result<Self> {
        use crate::error::KenshinError;

        let contents = std::fs::read_to_string(path).map_err(|e| KenshinError::ConfigError {
            message: format!("Failed to read config file: {}", e),
        })?;

        let config: Config =
            serde_yaml::from_str(&contents).map_err(|e| KenshinError::ConfigError {
                message: format!("Failed to parse config file: {}", e),
            })?;

        config.validate()?;

        Ok(config)
    }

    /// Validate configuration values
    fn validate(&self) -> crate::error::Result<()> {
        use crate::error::KenshinError;

        if self.formatting.line_length < 40 || self.formatting.line_length > 500 {
            return Err(KenshinError::ConfigError {
                message: format!(
                    "line_length must be between 40 and 500, got {}",
                    self.formatting.line_length
                ),
            });
        }

        if self.formatting.indent_width < 1 || self.formatting.indent_width > 8 {
            return Err(KenshinError::ConfigError {
                message: format!(
                    "indent_width must be between 1 and 8, got {}",
                    self.formatting.indent_width
                ),
            });
        }

        Ok(())
    }

    /// Get the indent string based on configuration
    #[cfg(test)]
    pub fn indent_string(&self) -> String {
        match self.formatting.indent_style {
            IndentStyle::Spaces => " ".repeat(self.formatting.indent_width),
            IndentStyle::Tabs => "\t".to_string(),
        }
    }

    /// Check if a file path should be included based on include/exclude patterns
    #[cfg(test)]
    pub fn should_include(&self, path: &std::path::Path) -> bool {
        use globset::{Glob, GlobSetBuilder};

        let path_str = path.to_string_lossy();

        // Check exclude patterns first
        let mut exclude_builder = GlobSetBuilder::new();
        for pattern in &self.exclude {
            if let Ok(glob) = Glob::new(pattern) {
                exclude_builder.add(glob);
            }
        }

        if let Ok(exclude_set) = exclude_builder.build() {
            if exclude_set.is_match(&*path_str) {
                return false;
            }
        }

        // Check include patterns
        let mut include_builder = GlobSetBuilder::new();
        for pattern in &self.include {
            if let Ok(glob) = Glob::new(pattern) {
                include_builder.add(glob);
            }
        }

        if let Ok(include_set) = include_builder.build() {
            return include_set.is_match(&*path_str);
        }

        false
    }
}

fn first_candidate_in(dir: &Path) -> Option<PathBuf> {
    CONFIG_FILE_NAMES
        .iter()
        .map(|name| dir.join(name))
        .find(|path| path.exists())
}

fn file_mtime(path: &Path) -> Option<SystemTime> {
    std::fs::metadata(path).and_then(|m| m.modified()).ok()
}

/// Cheap per-call re-validation replacing the full walk: stat the cwd
/// candidates (catches a config newly created where formatting runs) and the
/// discovered file's mtime (catches edits; a vanished file forces a re-walk).
/// A config newly created outside cwd (parent directory or home) is not
/// detected until something else invalidates the cache.
fn cache_is_fresh(cwd: &Path, cache: &DiscoveryCache) -> bool {
    let cwd_candidate = first_candidate_in(cwd);
    match &cache.found {
        Some((path, mtime)) => {
            if cwd_candidate.is_some_and(|candidate| candidate != *path) {
                return false;
            }
            file_mtime(path) == Some(*mtime)
        }
        None => cwd_candidate.is_none(),
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            version: "1.0".to_string(),
            parser: ParserConfig::default(),
            formatting: FormattingConfig::default(),
            include: vec!["**/*.rb".to_string(), "**/*.rake".to_string()],
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
    use crate::error::KenshinError;
    use std::io::Write;
    use std::path::Path;
    use tempfile::NamedTempFile;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.version, "1.0");
        assert_eq!(config.formatting.line_length, 100);
        assert_eq!(config.formatting.indent_width, 2);
        assert!(matches!(
            config.formatting.indent_style,
            IndentStyle::Spaces
        ));
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
        if let Err(KenshinError::ConfigError { message, .. }) = result {
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
        if let Err(KenshinError::ConfigError { message, .. }) = result {
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
        if let Err(KenshinError::ConfigError { message, .. }) = result {
            assert!(message.contains("parse"));
        }
    }

    // The discovery cache is process-global; serialize the tests that touch it.
    static CACHE_TEST_LOCK: Mutex<()> = Mutex::new(());

    fn write_indent_config(path: &Path, indent_width: usize) {
        std::fs::write(
            path,
            format!("formatting:\n  indent_width: {indent_width}\n"),
        )
        .unwrap();
    }

    #[test]
    fn test_resolve_explicit_missing_path_errors_loudly() {
        let result = Config::resolve(Some(Path::new("/nonexistent/kenshin.yml")));
        assert!(result.is_err());
    }

    #[test]
    fn test_resolve_explicit_broken_file_errors_loudly() {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"formatting:\n  line_length: not_a_number\n")
            .unwrap();
        file.flush().unwrap();

        assert!(Config::resolve(Some(file.path())).is_err());
    }

    #[test]
    fn test_resolve_explicit_path_cached_and_reloaded_on_change() {
        let _lock = CACHE_TEST_LOCK.lock().unwrap();
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("custom.yml");

        write_indent_config(&path, 4);
        assert_eq!(
            Config::resolve(Some(&path))
                .unwrap()
                .formatting
                .indent_width,
            4
        );

        std::thread::sleep(std::time::Duration::from_millis(20));
        write_indent_config(&path, 3);
        assert_eq!(
            Config::resolve(Some(&path))
                .unwrap()
                .formatting
                .indent_width,
            3
        );
    }

    #[test]
    fn test_discovered_broken_file_falls_back_to_defaults() {
        let _lock = CACHE_TEST_LOCK.lock().unwrap();
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("kenshin.yml"),
            "formatting:\n  indent_width: 99\n",
        )
        .unwrap();

        // Unlike an explicit path, discovery swallows load errors.
        let config = Config::discover_cached_from(dir.path());
        assert_eq!(config.formatting.indent_width, 2);
    }

    #[test]
    fn test_discovery_cache_reloads_on_mtime_change() {
        let _lock = CACHE_TEST_LOCK.lock().unwrap();
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("kenshin.yml");

        write_indent_config(&path, 4);
        assert_eq!(
            Config::discover_cached_from(dir.path())
                .formatting
                .indent_width,
            4
        );

        std::thread::sleep(std::time::Duration::from_millis(20));
        write_indent_config(&path, 3);
        assert_eq!(
            Config::discover_cached_from(dir.path())
                .formatting
                .indent_width,
            3
        );
    }

    #[test]
    fn test_discovery_cache_missing_file_falls_back_to_walk() {
        let _lock = CACHE_TEST_LOCK.lock().unwrap();
        let dir = tempfile::tempdir().unwrap();
        let sub = dir.path().join("sub");
        std::fs::create_dir(&sub).unwrap();
        let path = dir.path().join("kenshin.yml");

        write_indent_config(&path, 4);
        assert_eq!(
            Config::discover_cached_from(&sub).formatting.indent_width,
            4
        );

        std::fs::remove_file(&path).unwrap();
        assert_eq!(
            Config::discover_cached_from(&sub).formatting.indent_width,
            2
        );
    }

    #[test]
    fn test_discovery_nothing_found_cached_then_new_config_in_cwd() {
        let _lock = CACHE_TEST_LOCK.lock().unwrap();
        let dir = tempfile::tempdir().unwrap();

        assert_eq!(
            Config::discover_cached_from(dir.path())
                .formatting
                .indent_width,
            2
        );

        write_indent_config(&dir.path().join("kenshin.yml"), 5);
        assert_eq!(
            Config::discover_cached_from(dir.path())
                .formatting
                .indent_width,
            5
        );
    }

    #[test]
    fn test_discovery_accepts_legacy_rfmt_name() {
        let _lock = CACHE_TEST_LOCK.lock().unwrap();
        let dir = tempfile::tempdir().unwrap();

        write_indent_config(&dir.path().join(".rfmt.yml"), 6);
        assert_eq!(
            Config::discover_cached_from(dir.path())
                .formatting
                .indent_width,
            6
        );
    }

    #[test]
    fn test_discovery_prefers_kenshin_name_over_legacy() {
        let _lock = CACHE_TEST_LOCK.lock().unwrap();
        let dir = tempfile::tempdir().unwrap();

        write_indent_config(&dir.path().join("rfmt.yml"), 3);
        write_indent_config(&dir.path().join("kenshin.yml"), 4);
        assert_eq!(
            Config::discover_cached_from(dir.path())
                .formatting
                .indent_width,
            4
        );
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
        assert!(matches!(
            config.formatting.indent_style,
            IndentStyle::Spaces
        )); // default
    }
}
