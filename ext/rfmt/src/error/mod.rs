use magnus::{Error as MagnusError, Ruby};
use thiserror::Error;

pub type Result<T> = std::result::Result<T, RfmtError>;

#[derive(Error, Debug)]
pub enum RfmtError {
    #[error("Prism integration error: {0}")]
    PrismError(String),

    // Message-only kinds: lib/rfmt.rb rewrites these into its public
    // exception classes by their [Rfmt::...] prefix, so Display must not
    // add any wrapper text around the message.
    #[error("{0}")]
    ParseError(String),

    #[error("{0}")]
    ValidationError(String),

    #[error("{message}")]
    ConfigError { message: String },

    #[error("Format error: {0}")]
    FormatError(String),

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
            RfmtError::PrismError(_) => "PrismError",
            RfmtError::ParseError(_) => "ParseError",
            RfmtError::ValidationError(_) => "ValidationError",
            RfmtError::FormatError(_) => "FormatError",
            RfmtError::UnsupportedFeature { .. } => "UnsupportedFeature",
            RfmtError::ConfigError { .. } => "ConfigError",
        };

        MagnusError::new(
            ruby.exception_standard_error(),
            format!("[Rfmt::{}] {}", exception_class, self),
        )
    }
}
