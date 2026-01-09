use crate::error::{Result, RfmtError};

/// Validate source code size
pub fn validate_source_size(source: &str, max_size: u64) -> Result<()> {
    let size = source.len() as u64;

    if size > max_size {
        return Err(RfmtError::UnsupportedFeature {
            feature: "Large file".to_string(),
            explanation: format!(
                "Source code is too large ({} bytes, max {} bytes)",
                size, max_size
            ),
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_source_size_ok() {
        assert!(validate_source_size("small", 1000).is_ok());
    }

    #[test]
    fn test_validate_source_size_at_limit() {
        let source = "a".repeat(1000);
        assert!(validate_source_size(&source, 1000).is_ok());
    }

    #[test]
    fn test_validate_source_size_exceeds_limit() {
        let source = "a".repeat(1001);
        assert!(validate_source_size(&source, 1000).is_err());
    }

    #[test]
    fn test_validate_source_size_empty() {
        assert!(validate_source_size("", 1000).is_ok());
    }

    #[test]
    fn test_validate_source_size_unicode() {
        // "日本語" = 9 bytes in UTF-8
        let source = "日本語";
        assert!(validate_source_size(source, 9).is_ok());
        assert!(validate_source_size(source, 8).is_err());
    }
}
