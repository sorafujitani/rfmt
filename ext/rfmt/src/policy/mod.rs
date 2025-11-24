pub mod limits;
pub mod validation;

use crate::error::{Result, RfmtError};
use std::path::{Path, PathBuf};

/// Security policy for rfmt operations
///
/// Defines limits and validation rules to ensure safe operation
/// and prevent resource exhaustion or malicious inputs.
#[derive(Debug, Clone)]
pub struct SecurityPolicy {
    /// Maximum file size in bytes (default: 10MB)
    pub max_file_size: u64,

    /// Maximum memory usage in bytes (default: 100MB)
    pub max_memory_usage: usize,

    /// Timeout for operations in seconds (default: 30s)
    pub timeout_seconds: u64,

    /// Maximum recursion depth for AST traversal (default: 1000)
    pub max_recursion_depth: usize,

    /// Maximum number of parallel threads (default: num_cpus)
    pub max_threads: usize,

    /// Allow symbolic links (default: false)
    pub allow_symlinks: bool,
}

impl SecurityPolicy {
    /// Create a new security policy with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a strict security policy with tighter limits
    pub fn strict() -> Self {
        Self {
            max_file_size: 5 * 1024 * 1024,     // 5MB
            max_memory_usage: 50 * 1024 * 1024, // 50MB
            timeout_seconds: 15,
            max_recursion_depth: 500,
            max_threads: 2,
            allow_symlinks: false,
        }
    }

    /// Create a permissive security policy with relaxed limits
    pub fn permissive() -> Self {
        Self {
            max_file_size: 50 * 1024 * 1024,     // 50MB
            max_memory_usage: 500 * 1024 * 1024, // 500MB
            timeout_seconds: 120,
            max_recursion_depth: 5000,
            max_threads: num_cpus::get(),
            allow_symlinks: true,
        }
    }

    /// Validate a file path according to the policy
    pub fn validate_path(&self, path: &Path) -> Result<PathBuf> {
        validation::validate_path(path, self)
    }

    /// Validate file size according to the policy
    pub fn validate_file_size(&self, path: &Path) -> Result<()> {
        limits::validate_file_size(path, self.max_file_size)
    }

    /// Check if a recursion depth is within limits
    pub fn check_recursion_depth(&self, depth: usize) -> Result<()> {
        limits::check_recursion_depth(depth, self.max_recursion_depth)
    }

    /// Validate source code size
    pub fn validate_source_size(&self, source: &str) -> Result<()> {
        validation::validate_source_size(source, self.max_file_size)
    }

    /// Perform all validations for a file
    pub fn validate_file(&self, path: &Path) -> Result<PathBuf> {
        // Validate and sanitize path
        let canonical_path = self.validate_path(path)?;

        // Validate file size
        self.validate_file_size(&canonical_path)?;

        Ok(canonical_path)
    }
}

impl Default for SecurityPolicy {
    fn default() -> Self {
        Self {
            max_file_size: 10 * 1024 * 1024,     // 10MB
            max_memory_usage: 100 * 1024 * 1024, // 100MB
            timeout_seconds: 30,
            max_recursion_depth: 1000,
            max_threads: num_cpus::get().min(4), // Max 4 threads by default
            allow_symlinks: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_policy() {
        let policy = SecurityPolicy::default();
        assert_eq!(policy.max_file_size, 10 * 1024 * 1024);
        assert_eq!(policy.timeout_seconds, 30);
        assert_eq!(policy.max_recursion_depth, 1000);
        assert!(!policy.allow_symlinks);
    }

    #[test]
    fn test_strict_policy() {
        let policy = SecurityPolicy::strict();
        assert_eq!(policy.max_file_size, 5 * 1024 * 1024);
        assert_eq!(policy.timeout_seconds, 15);
        assert_eq!(policy.max_recursion_depth, 500);
        assert!(!policy.allow_symlinks);
    }

    #[test]
    fn test_permissive_policy() {
        let policy = SecurityPolicy::permissive();
        assert_eq!(policy.max_file_size, 50 * 1024 * 1024);
        assert_eq!(policy.timeout_seconds, 120);
        assert_eq!(policy.max_recursion_depth, 5000);
        assert!(policy.allow_symlinks);
    }
}
