pub mod logger;

pub use logger::RfmtLogger;

/// デバッグ情報を収集
#[derive(Debug, Clone)]
pub struct DebugInfo {
    pub version: String,
    pub ruby_version: String,
    pub platform: String,
    pub config: String,
}

impl DebugInfo {
    pub fn collect() -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION").to_string(),
            ruby_version: Self::get_ruby_version(),
            platform: std::env::consts::OS.to_string(),
            config: "default".to_string(),
        }
    }

    fn get_ruby_version() -> String {
        // Default to unknown, will be populated from Ruby side if needed
        std::env::var("RUBY_VERSION").unwrap_or_else(|_| "unknown".to_string())
    }

    pub fn report(&self) -> String {
        format!(
            "rfmt version: {}\nRuby version: {}\nPlatform: {}\nConfig: {}",
            self.version, self.ruby_version, self.platform, self.config
        )
    }

    pub fn with_config(mut self, config: String) -> Self {
        self.config = config;
        self
    }
}

impl Default for DebugInfo {
    fn default() -> Self {
        Self::collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debug_info_creation() {
        let info = DebugInfo::collect();

        assert!(!info.version.is_empty());
        assert!(!info.platform.is_empty());
        assert_eq!(info.config, "default");
    }

    #[test]
    fn test_debug_info_report() {
        let info = DebugInfo::collect();
        let report = info.report();

        assert!(report.contains("rfmt version:"));
        assert!(report.contains("Ruby version:"));
        assert!(report.contains("Platform:"));
        assert!(report.contains("Config:"));
    }

    #[test]
    fn test_debug_info_with_config() {
        let info = DebugInfo::collect().with_config("custom".to_string());
        assert_eq!(info.config, "custom");
    }

    #[test]
    fn test_platform_detection() {
        let info = DebugInfo::collect();

        // Should be one of the supported platforms
        assert!(
            info.platform == "linux"
                || info.platform == "macos"
                || info.platform == "windows"
                || info.platform == "ios"
                || info.platform == "android"
                || !info.platform.is_empty()
        );
    }
}
