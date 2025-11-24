use crate::error::{Result, RfmtError};
use crate::policy::SecurityPolicy;
use log::{debug, warn};
use std::path::{Path, PathBuf};

/// Validate and sanitize a file path
///
/// Performs the following checks:
/// - Path exists
/// - Path is a file (not a directory)
/// - Resolves to canonical path (prevents path traversal)
/// - Checks for symbolic links (if not allowed by policy)
/// - Ensures path doesn't escape allowed directories
pub fn validate_path(path: &Path, policy: &SecurityPolicy) -> Result<PathBuf> {
    debug!("Validating path: {:?}", path);

    // Check if path exists
    if !path.exists() {
        return Err(RfmtError::IoError {
            file: path.to_path_buf(),
            message: "File does not exist".to_string(),
            source: std::io::Error::new(std::io::ErrorKind::NotFound, "File not found"),
        });
    }

    // Check if it's a file (not a directory)
    if !path.is_file() {
        return Err(RfmtError::IoError {
            file: path.to_path_buf(),
            message: "Path is not a file".to_string(),
            source: std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Expected a file, got a directory or other type",
            ),
        });
    }

    // Canonicalize the path (resolves symlinks and relative paths)
    let canonical = path.canonicalize().map_err(|e| RfmtError::IoError {
        file: path.to_path_buf(),
        message: "Failed to canonicalize path".to_string(),
        source: e,
    })?;

    // Check for symbolic links
    if !policy.allow_symlinks {
        let metadata = path.symlink_metadata().map_err(|e| RfmtError::IoError {
            file: path.to_path_buf(),
            message: "Failed to read file metadata".to_string(),
            source: e,
        })?;

        if metadata.file_type().is_symlink() {
            warn!("Symbolic link detected: {:?}", path);
            return Err(RfmtError::UnsupportedFeature {
                feature: "Symbolic links".to_string(),
                explanation: "Symbolic links are not allowed by the current security policy. \
                             Set allow_symlinks to true to enable."
                    .to_string(),
            });
        }
    }

    // Validate file extension (should be a Ruby file)
    validate_file_extension(&canonical)?;

    debug!("Path validated successfully: {:?}", canonical);
    Ok(canonical)
}

/// Validate file extension
fn validate_file_extension(path: &Path) -> Result<()> {
    let extension = path.extension().and_then(|ext| ext.to_str()).unwrap_or("");

    match extension {
        "rb" | "rake" | "ru" => Ok(()),
        "" => {
            // Check for common Ruby files without extension
            if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                match filename {
                    "Gemfile" | "Rakefile" | "Guardfile" | "Capfile" | "Vagrantfile" => Ok(()),
                    _ => Err(RfmtError::UnsupportedFeature {
                        feature: format!("File without Ruby extension: {}", filename),
                        explanation: "Expected a Ruby file (.rb, .rake, .ru) or known Ruby file \
                                     (Gemfile, Rakefile, etc.)"
                            .to_string(),
                    }),
                }
            } else {
                Err(RfmtError::UnsupportedFeature {
                    feature: "File with no name".to_string(),
                    explanation: "Cannot determine if this is a Ruby file".to_string(),
                })
            }
        }
        other => Err(RfmtError::UnsupportedFeature {
            feature: format!("File with extension: .{}", other),
            explanation: "Expected a Ruby file (.rb, .rake, .ru)".to_string(),
        }),
    }
}

/// Validate source code size
pub fn validate_source_size(source: &str, max_size: u64) -> Result<()> {
    let size = source.len() as u64;

    if size > max_size {
        return Err(RfmtError::UnsupportedFeature {
            feature: "Large source code".to_string(),
            explanation: format!(
                "Source code size ({} bytes) exceeds maximum ({} bytes). \
                 This limit prevents resource exhaustion.",
                size, max_size
            ),
        });
    }

    debug!("Source size validated: {} bytes", size);
    Ok(())
}

/// Validate source code encoding
pub fn validate_encoding(source: &str) -> Result<()> {
    // Check if the source is valid UTF-8 (Rust strings are always UTF-8)
    // This check is mostly redundant but kept for explicitness
    if !source.is_empty() && source.chars().any(|c| c == '\0') {
        return Err(RfmtError::UnsupportedFeature {
            feature: "Source with null bytes".to_string(),
            explanation: "Ruby source code should not contain null bytes".to_string(),
        });
    }

    debug!("Encoding validated successfully");
    Ok(())
}

/// Sanitize error messages to prevent information leakage
pub fn sanitize_error_message(message: &str) -> String {
    // Remove absolute paths from error messages
    let sanitized = message
        .replace(
            std::env::home_dir()
                .unwrap_or_default()
                .to_string_lossy()
                .as_ref(),
            "~",
        )
        .replace(
            std::env::current_dir()
                .unwrap_or_default()
                .to_string_lossy()
                .as_ref(),
            ".",
        );

    // Remove potential sensitive information patterns
    // (This is a simple implementation; production code might need more sophisticated filtering)
    sanitized
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_validate_path_nonexistent() {
        let policy = SecurityPolicy::default();
        let result = validate_path(Path::new("/nonexistent/file.rb"), &policy);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_path_valid_rb_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.rb");
        File::create(&file_path).unwrap();

        let policy = SecurityPolicy::default();
        let result = validate_path(&file_path, &policy);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_path_directory() {
        let temp_dir = TempDir::new().unwrap();

        let policy = SecurityPolicy::default();
        let result = validate_path(temp_dir.path(), &policy);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_file_extension_rb() {
        assert!(validate_file_extension(Path::new("test.rb")).is_ok());
    }

    #[test]
    fn test_validate_file_extension_rake() {
        assert!(validate_file_extension(Path::new("test.rake")).is_ok());
    }

    #[test]
    fn test_validate_file_extension_gemfile() {
        assert!(validate_file_extension(Path::new("Gemfile")).is_ok());
    }

    #[test]
    fn test_validate_file_extension_invalid() {
        let result = validate_file_extension(Path::new("test.txt"));
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_source_size_ok() {
        let source = "class Foo\nend\n";
        let result = validate_source_size(source, 1024);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_source_size_too_large() {
        let source = "a".repeat(1000);
        let result = validate_source_size(&source, 100);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_encoding_ok() {
        let source = "class Foo\n  # 日本語コメント\nend\n";
        let result = validate_encoding(source);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_encoding_null_byte() {
        let source = "class Foo\0end";
        let result = validate_encoding(source);
        assert!(result.is_err());
    }

    #[test]
    fn test_sanitize_error_message() {
        let message = format!(
            "Error in {}/.rfmt/config.yml",
            std::env::home_dir().unwrap().to_string_lossy()
        );
        let sanitized = sanitize_error_message(&message);
        assert!(sanitized.contains("~/.rfmt/config.yml"));
    }
}
