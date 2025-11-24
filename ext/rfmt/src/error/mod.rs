pub mod context;
pub mod handler;

pub use context::ErrorContext;
pub use handler::ErrorHandler;

use magnus::{Error as MagnusError, Ruby};
use std::path::PathBuf;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, RfmtError>;

#[derive(Error, Debug)]
pub enum RfmtError {
    #[error("Parse error in {file}:{line}:{column}\n{message}\n{snippet}")]
    ParseError {
        file: PathBuf,
        line: usize,
        column: usize,
        message: String,
        snippet: String, // コードスニペット
    },

    #[error("Configuration error: {message}\nFile: {file}\nSuggestion: {suggestion}")]
    ConfigError {
        message: String,
        file: PathBuf,
        suggestion: String,
    },

    #[error("Prism integration error: {0}")]
    PrismError(String),

    #[error("IO error for file {file}: {message}")]
    IoError {
        file: PathBuf,
        message: String,
        #[source]
        source: std::io::Error,
    },

    #[error("Formatting error: {message}\nNode: {node_type} at {location}")]
    FormattingError {
        message: String,
        node_type: String,
        location: String,
    },

    #[error("Formatting rule error in {rule}: {message}")]
    RuleError { rule: String, message: String },

    #[error("Format error: {0}")]
    FormatError(String),

    #[error("Internal error: {message}\nPlease report this as a bug")]
    InternalError {
        message: String,
        backtrace: String,
    },

    #[error("Unsupported feature: {feature}\n{explanation}")]
    UnsupportedFeature {
        feature: String,
        explanation: String,
    },
}

// Implement From for std::fmt::Error
impl From<std::fmt::Error> for RfmtError {
    fn from(err: std::fmt::Error) -> Self {
        RfmtError::FormatError(err.to_string())
    }
}

impl RfmtError {
    /// Convert RfmtError to Magnus Error for Ruby interop
    pub fn to_magnus_error(&self, ruby: &Ruby) -> MagnusError {
        let exception_class = match self {
            RfmtError::ParseError { .. } => "ParseError",
            RfmtError::ConfigError { .. } => "ConfigError",
            RfmtError::PrismError(_) => "PrismError",
            RfmtError::IoError { .. } => "IOError",
            RfmtError::RuleError { .. } => "RuleError",
            RfmtError::FormatError(_) => "FormatError",
            RfmtError::FormattingError { .. } => "FormattingError",
            RfmtError::InternalError { .. } => "InternalError",
            RfmtError::UnsupportedFeature { .. } => "UnsupportedFeature",
        };

        MagnusError::new(
            ruby.exception_standard_error(),
            format!("[Rfmt::{}] {}", exception_class, self),
        )
    }

    /// ユーザーフレンドリーなエラーメッセージを生成
    pub fn user_message(&self) -> String {
        match self {
            RfmtError::ParseError { file, line, column, message, snippet } => {
                format!(
                    "Parse error in {}:{}:{}\n{}\n\nCode:\n{}",
                    file.display(),
                    line,
                    column,
                    message,
                    snippet
                )
            }
            RfmtError::ConfigError { message, file, suggestion } => {
                format!(
                    "Configuration error: {}\nFile: {}\n\nSuggestion: {}",
                    message,
                    file.display(),
                    suggestion
                )
            }
            RfmtError::FormattingError { message, node_type, location } => {
                format!(
                    "Formatting error: {}\nNode type: {}\nLocation: {}",
                    message, node_type, location
                )
            }
            RfmtError::InternalError { message, backtrace } => {
                format!(
                    "Internal error: {}\n\nBacktrace:\n{}\n\nPlease report this as a bug at: https://github.com/fujitanisora/rfmt/issues",
                    message, backtrace
                )
            }
            RfmtError::UnsupportedFeature { feature, explanation } => {
                format!(
                    "Unsupported feature: {}\n\n{}",
                    feature, explanation
                )
            }
            _ => self.to_string(),
        }
    }

    /// エラーコードを返す（ドキュメント参照用）
    pub fn error_code(&self) -> &'static str {
        match self {
            RfmtError::ParseError { .. } => "E001",
            RfmtError::ConfigError { .. } => "E002",
            RfmtError::IoError { .. } => "E003",
            RfmtError::FormattingError { .. } => "E004",
            RfmtError::RuleError { .. } => "E005",
            RfmtError::PrismError(_) => "E007",
            RfmtError::FormatError(_) => "E008",
            RfmtError::UnsupportedFeature { .. } => "E006",
            RfmtError::InternalError { .. } => "E999",
        }
    }

    /// ヘルプURLを返す
    pub fn help_url(&self) -> String {
        format!("https://rfmt.dev/errors/{}", self.error_code())
    }
}

/// エラーリカバリー戦略
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryStrategy {
    /// エラーのあるノードをスキップ
    Skip,
    /// 元のフォーマットを保持
    PreserveOriginal,
    /// 最小限のフォーマットのみ適用
    MinimalFormat,
    /// 処理を中断
    Abort,
}
