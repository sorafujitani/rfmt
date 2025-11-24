use super::super::error::{RecoveryStrategy, RfmtError};

pub struct ErrorHandler {
    strategy: RecoveryStrategy,
    errors: Vec<RfmtError>,
    max_errors: usize,
}

impl ErrorHandler {
    pub fn new(strategy: RecoveryStrategy) -> Self {
        Self {
            strategy,
            errors: Vec::new(),
            max_errors: 100, // 最大エラー数
        }
    }

    pub fn with_max_errors(mut self, max: usize) -> Self {
        self.max_errors = max;
        self
    }

    pub fn handle(&mut self, error: RfmtError) -> Option<String> {
        self.errors.push(error);

        if self.errors.len() >= self.max_errors {
            eprintln!("Too many errors ({}). Aborting.", self.max_errors);
            return None;
        }

        match self.strategy {
            RecoveryStrategy::Skip => Some(String::new()),
            RecoveryStrategy::PreserveOriginal => Some(String::from("/* preserved */")),
            RecoveryStrategy::MinimalFormat => Some(String::from("/* minimal */")),
            RecoveryStrategy::Abort => None,
        }
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn errors(&self) -> &[RfmtError] {
        &self.errors
    }

    pub fn error_count(&self) -> usize {
        self.errors.len()
    }

    pub fn report(&self) -> String {
        if self.errors.is_empty() {
            return String::from("No errors encountered.");
        }

        let mut report = format!("Encountered {} error(s):\n\n", self.errors.len());

        for (i, error) in self.errors.iter().enumerate() {
            report.push_str(&format!(
                "{}. [{}] {}\n   Help: {}\n\n",
                i + 1,
                error.error_code(),
                error.user_message(),
                error.help_url()
            ));
        }

        report
    }

    pub fn clear(&mut self) {
        self.errors.clear();
    }
}

impl Default for ErrorHandler {
    fn default() -> Self {
        Self::new(RecoveryStrategy::Abort)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_error_handler_creation() {
        let handler = ErrorHandler::new(RecoveryStrategy::Skip);
        assert_eq!(handler.error_count(), 0);
        assert!(!handler.has_errors());
    }

    #[test]
    fn test_handle_with_skip_strategy() {
        let mut handler = ErrorHandler::new(RecoveryStrategy::Skip);
        let error = RfmtError::FormatError("test error".to_string());

        let result = handler.handle(error);
        assert_eq!(result, Some(String::new()));
        assert_eq!(handler.error_count(), 1);
        assert!(handler.has_errors());
    }

    #[test]
    fn test_handle_with_abort_strategy() {
        let mut handler = ErrorHandler::new(RecoveryStrategy::Abort);
        let error = RfmtError::FormatError("test error".to_string());

        let result = handler.handle(error);
        assert_eq!(result, None);
        assert_eq!(handler.error_count(), 1);
    }

    #[test]
    fn test_max_errors_limit() {
        let mut handler = ErrorHandler::new(RecoveryStrategy::Skip).with_max_errors(3);

        // First 2 errors should be handled
        for i in 0..2 {
            let error = RfmtError::FormatError(format!("error {}", i));
            let result = handler.handle(error);
            assert!(result.is_some(), "Error {} should be handled", i);
        }

        // 3rd error should abort (because errors.len() == 2, then push makes it 3, which == max_errors)
        let error = RfmtError::FormatError("error 2".to_string());
        let result = handler.handle(error);
        assert!(result.is_none(), "Should abort at max errors");
        assert_eq!(handler.error_count(), 3);
    }

    #[test]
    fn test_error_report() {
        let mut handler = ErrorHandler::new(RecoveryStrategy::Skip);

        let error1 = RfmtError::FormatError("first error".to_string());
        let error2 = RfmtError::PrismError("second error".to_string());

        handler.handle(error1);
        handler.handle(error2);

        let report = handler.report();
        assert!(report.contains("Encountered 2 error(s)"));
        assert!(report.contains("first error"));
        assert!(report.contains("second error"));
        assert!(report.contains("[E008]")); // FormatError code
        assert!(report.contains("[E007]")); // PrismError code
    }

    #[test]
    fn test_clear_errors() {
        let mut handler = ErrorHandler::new(RecoveryStrategy::Skip);
        let error = RfmtError::FormatError("test".to_string());

        handler.handle(error);
        assert!(handler.has_errors());

        handler.clear();
        assert!(!handler.has_errors());
        assert_eq!(handler.error_count(), 0);
    }

    #[test]
    fn test_recovery_strategies() {
        let strategies = vec![
            (RecoveryStrategy::Skip, Some(String::new())),
            (RecoveryStrategy::PreserveOriginal, Some(String::from("/* preserved */"))),
            (RecoveryStrategy::MinimalFormat, Some(String::from("/* minimal */"))),
            (RecoveryStrategy::Abort, None),
        ];

        for (strategy, expected) in strategies {
            let mut handler = ErrorHandler::new(strategy);
            let error = RfmtError::FormatError("test".to_string());
            let result = handler.handle(error);
            assert_eq!(result, expected, "Strategy {:?} failed", strategy);
        }
    }
}
