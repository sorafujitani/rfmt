use magnus::{Error as MagnusError, Ruby};
use thiserror::Error;

pub type Result<T> = std::result::Result<T, KenshinError>;

#[derive(Error, Debug)]
pub enum KenshinError {
    #[error("Prism integration error: {0}")]
    PrismError(String),

    // Message-only kinds: lib/kenshin.rb rewrites these into its public
    // exception classes by their [Kenshin::...] prefix, so Display must not
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
impl From<std::fmt::Error> for KenshinError {
    fn from(err: std::fmt::Error) -> Self {
        KenshinError::FormatError(err.to_string())
    }
}

impl KenshinError {
    /// Convert KenshinError to Magnus Error for Ruby interop
    pub fn to_magnus_error(&self, ruby: &Ruby) -> MagnusError {
        let exception_class = match self {
            KenshinError::PrismError(_) => "PrismError",
            KenshinError::ParseError(_) => "ParseError",
            KenshinError::ValidationError(_) => "ValidationError",
            KenshinError::FormatError(_) => "FormatError",
            KenshinError::UnsupportedFeature { .. } => "UnsupportedFeature",
            KenshinError::ConfigError { .. } => "ConfigError",
        };

        MagnusError::new(
            ruby.exception_standard_error(),
            format!("[Kenshin::{}] {}", exception_class, self),
        )
    }
}
