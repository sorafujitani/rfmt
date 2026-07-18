//! Output guard: never return syntactically invalid Ruby to the caller.
//! Moved here from lib/rfmt.rb's validate_output! at the phase-6 switchover.

use crate::error::{Result, RfmtError};

pub fn validate_output(formatted: &str) -> Result<()> {
    let bytes = formatted.as_bytes();
    let parse_result = ruby_prism::parse(bytes);

    let Some(error) = parse_result.errors().next() else {
        return Ok(());
    };

    let line = 1 + bytes[..error.location().start_offset()]
        .iter()
        .filter(|&&b| b == b'\n')
        .count();
    Err(RfmtError::ValidationError(format!(
        "Formatter produced syntactically invalid output (this is a bug in rfmt, not in your code): {} at line {}",
        error.message(),
        line
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_valid_ruby() {
        assert!(validate_output("x = 1\n").is_ok());
    }

    #[test]
    fn rejects_invalid_ruby_with_validation_error() {
        let err = validate_output("def broken(\n").unwrap_err();

        match err {
            RfmtError::ValidationError(message) => {
                assert!(
                    message.starts_with(
                        "Formatter produced syntactically invalid output (this is a bug in rfmt, not in your code): "
                    ),
                    "unexpected message: {message}"
                );
                assert!(
                    message.contains(" at line 1"),
                    "unexpected message: {message}"
                );
            }
            other => panic!("expected ValidationError, got {other:?}"),
        }
    }

    #[test]
    fn reports_the_line_of_the_first_error() {
        let err = validate_output("x = 1\ny = 2\ndef broken(\n").unwrap_err();

        match err {
            RfmtError::ValidationError(message) => {
                assert!(
                    message.contains(" at line 3"),
                    "unexpected message: {message}"
                );
            }
            other => panic!("expected ValidationError, got {other:?}"),
        }
    }
}
