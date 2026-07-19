pub mod validation;

use crate::error::Result;

/// Security policy for kenshin operations
#[derive(Debug, Clone)]
pub struct SecurityPolicy {
    /// Maximum file size in bytes (default: 10MB)
    pub max_file_size: u64,
}

impl SecurityPolicy {
    /// Validate source code size
    pub fn validate_source_size(&self, source: &str) -> Result<()> {
        validation::validate_source_size(source, self.max_file_size)
    }
}

impl Default for SecurityPolicy {
    fn default() -> Self {
        Self {
            max_file_size: 10 * 1024 * 1024, // 10MB
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
    }
}
