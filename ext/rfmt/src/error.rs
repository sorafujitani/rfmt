use magnus::{Error as MagnusError, Ruby};
use std::path::PathBuf;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, RfmtError>;

#[derive(Error, Debug)]
pub enum RfmtError {
    #[error("Parse error at {line}:{column}: {message}")]
    ParseError {
        line: usize,
        column: usize,
        message: String,
    },

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Prism integration error: {0}")]
    PrismError(String),

    #[error("IO error for file {file}: {source}")]
    IoError {
        file: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Formatting rule error in {rule}: {message}")]
    RuleError { rule: String, message: String },

    #[error("Format error: {0}")]
    FormatError(String),

    #[error("Internal error: {0}")]
    InternalError(String),
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
            RfmtError::ConfigError(_) => "ConfigError",
            RfmtError::PrismError(_) => "PrismError",
            RfmtError::IoError { .. } => "IOError",
            RfmtError::RuleError { .. } => "RuleError",
            RfmtError::FormatError(_) => "FormatError",
            RfmtError::InternalError(_) => "InternalError",
        };

        MagnusError::new(
            ruby.exception_standard_error(),
            format!("[Rfmt::{}] {}", exception_class, self),
        )
    }
}

/// Error recovery strategy
pub enum RecoveryStrategy {
    /// Skip the node with error
    SkipNode,
    /// Preserve original formatting
    PreserveOriginal,
    /// Apply minimal formatting only
    MinimalFormat,
    /// Abort processing
    Abort,
}

pub struct ErrorRecovery {
    pub strategy: RecoveryStrategy,
}

impl ErrorRecovery {
    pub fn new(strategy: RecoveryStrategy) -> Self {
        Self { strategy }
    }

    pub fn with_skip() -> Self {
        Self::new(RecoveryStrategy::SkipNode)
    }

    pub fn with_preserve() -> Self {
        Self::new(RecoveryStrategy::PreserveOriginal)
    }
}
