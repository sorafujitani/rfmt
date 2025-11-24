use crate::error::{Result, RfmtError};
use log::{debug, warn};
use std::cell::RefCell;
use std::path::Path;
use std::time::{Duration, Instant};

/// Validate file size against the maximum allowed
pub fn validate_file_size(path: &Path, max_size: u64) -> Result<()> {
    let metadata = std::fs::metadata(path).map_err(|e| RfmtError::IoError {
        file: path.to_path_buf(),
        message: "Failed to read file metadata".to_string(),
        source: e,
    })?;

    let file_size = metadata.len();

    if file_size > max_size {
        warn!(
            "File size ({} bytes) exceeds maximum ({} bytes): {:?}",
            file_size, max_size, path
        );
        return Err(RfmtError::UnsupportedFeature {
            feature: "Large file".to_string(),
            explanation: format!(
                "File size ({} bytes) exceeds maximum allowed size ({} bytes). \
                 Consider splitting the file or adjusting the max_file_size policy.",
                file_size, max_size
            ),
        });
    }

    debug!("File size validated: {} bytes", file_size);
    Ok(())
}

/// Check if recursion depth is within limits
pub fn check_recursion_depth(current_depth: usize, max_depth: usize) -> Result<()> {
    if current_depth > max_depth {
        warn!(
            "Recursion depth ({}) exceeds maximum ({})",
            current_depth, max_depth
        );
        return Err(RfmtError::InternalError {
            message: format!(
                "Maximum recursion depth ({}) exceeded. Current depth: {}",
                max_depth, current_depth
            ),
            backtrace: std::backtrace::Backtrace::capture().to_string(),
        });
    }

    Ok(())
}

/// Timeout guard for operations
pub struct TimeoutGuard {
    start: Instant,
    timeout: Duration,
    operation: String,
}

impl TimeoutGuard {
    /// Create a new timeout guard
    pub fn new(timeout_seconds: u64, operation: impl Into<String>) -> Self {
        Self {
            start: Instant::now(),
            timeout: Duration::from_secs(timeout_seconds),
            operation: operation.into(),
        }
    }

    /// Check if the operation has timed out
    pub fn check(&self) -> Result<()> {
        let elapsed = self.start.elapsed();

        if elapsed > self.timeout {
            warn!(
                "Operation '{}' timed out after {:?} (limit: {:?})",
                self.operation, elapsed, self.timeout
            );
            return Err(RfmtError::InternalError {
                message: format!(
                    "Operation '{}' timed out after {:?}. Maximum allowed time: {:?}",
                    self.operation, elapsed, self.timeout
                ),
                backtrace: String::new(),
            });
        }

        Ok(())
    }

    /// Get elapsed time
    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }

    /// Get remaining time
    pub fn remaining(&self) -> Duration {
        self.timeout.saturating_sub(self.start.elapsed())
    }
}

/// Memory usage tracker
pub struct MemoryTracker {
    initial_usage: usize,
    max_usage: usize,
}

impl MemoryTracker {
    /// Create a new memory tracker
    pub fn new(max_usage: usize) -> Self {
        Self {
            initial_usage: Self::current_usage(),
            max_usage,
        }
    }

    /// Get current memory usage (approximation)
    fn current_usage() -> usize {
        // This is a simplified implementation
        // In production, you might want to use a proper memory profiling library
        #[cfg(target_os = "linux")]
        {
            if let Ok(status) = std::fs::read_to_string("/proc/self/status") {
                for line in status.lines() {
                    if line.starts_with("VmRSS:") {
                        if let Some(kb) = line.split_whitespace().nth(1) {
                            if let Ok(kb_val) = kb.parse::<usize>() {
                                return kb_val * 1024; // Convert KB to bytes
                            }
                        }
                    }
                }
            }
        }

        // Fallback: return 0 (tracking disabled on non-Linux systems)
        0
    }

    /// Check if memory usage is within limits
    pub fn check(&self) -> Result<()> {
        let current = Self::current_usage();

        if current == 0 {
            // Memory tracking not available on this platform
            return Ok(());
        }

        let used = current.saturating_sub(self.initial_usage);

        if used > self.max_usage {
            warn!(
                "Memory usage ({} bytes) exceeds maximum ({} bytes)",
                used, self.max_usage
            );
            return Err(RfmtError::InternalError {
                message: format!(
                    "Memory usage ({} bytes) exceeds maximum allowed ({} bytes)",
                    used, self.max_usage
                ),
                backtrace: String::new(),
            });
        }

        debug!("Memory usage: {} bytes", used);
        Ok(())
    }

    /// Get current memory usage delta
    pub fn current_delta(&self) -> usize {
        Self::current_usage().saturating_sub(self.initial_usage)
    }
}

/// Recursion depth tracker
pub struct RecursionTracker {
    depth: RefCell<usize>,
    max_depth: usize,
}

impl RecursionTracker {
    /// Create a new recursion tracker
    pub fn new(max_depth: usize) -> Self {
        Self {
            depth: RefCell::new(0),
            max_depth,
        }
    }

    /// Enter a new recursion level
    pub fn enter(&self) -> Result<RecursionGuard<'_>> {
        let mut depth = self.depth.borrow_mut();
        *depth += 1;
        let current_depth = *depth;
        drop(depth); // Release borrow before calling check_recursion_depth

        check_recursion_depth(current_depth, self.max_depth)?;
        debug!("Recursion depth: {}", current_depth);
        Ok(RecursionGuard { tracker: self })
    }

    /// Get current depth
    pub fn depth(&self) -> usize {
        *self.depth.borrow()
    }
}

/// RAII guard for recursion tracking
pub struct RecursionGuard<'a> {
    tracker: &'a RecursionTracker,
}

impl<'a> Drop for RecursionGuard<'a> {
    fn drop(&mut self) {
        let mut depth = self.tracker.depth.borrow_mut();
        *depth = depth.saturating_sub(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_validate_file_size_ok() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.rb");
        let mut file = File::create(&file_path).unwrap();
        file.write_all(b"class Foo\nend\n").unwrap();

        let result = validate_file_size(&file_path, 1024);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_file_size_too_large() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.rb");
        let mut file = File::create(&file_path).unwrap();
        file.write_all(&vec![b'a'; 1000]).unwrap();

        let result = validate_file_size(&file_path, 100);
        assert!(result.is_err());
    }

    #[test]
    fn test_check_recursion_depth_ok() {
        let result = check_recursion_depth(10, 100);
        assert!(result.is_ok());
    }

    #[test]
    fn test_check_recursion_depth_exceeded() {
        let result = check_recursion_depth(150, 100);
        assert!(result.is_err());
    }

    #[test]
    fn test_timeout_guard_not_expired() {
        let guard = TimeoutGuard::new(10, "test");
        let result = guard.check();
        assert!(result.is_ok());
        assert!(guard.elapsed() < Duration::from_secs(1));
    }

    #[test]
    fn test_timeout_guard_expired() {
        let guard = TimeoutGuard::new(0, "test");
        std::thread::sleep(Duration::from_millis(100));
        let result = guard.check();
        assert!(result.is_err());
    }

    #[test]
    fn test_memory_tracker_basic() {
        let tracker = MemoryTracker::new(1024 * 1024 * 100); // 100MB
        let result = tracker.check();
        // Should succeed unless we're somehow using > 100MB
        assert!(result.is_ok() || cfg!(not(target_os = "linux")));
    }

    #[test]
    fn test_recursion_tracker() {
        let tracker = RecursionTracker::new(10);
        assert_eq!(tracker.depth(), 0);

        {
            let _guard1 = tracker.enter().unwrap();
            assert_eq!(tracker.depth(), 1);

            {
                let _guard2 = tracker.enter().unwrap();
                assert_eq!(tracker.depth(), 2);
            }

            assert_eq!(tracker.depth(), 1);
        }

        assert_eq!(tracker.depth(), 0);
    }

    #[test]
    fn test_recursion_tracker_exceeds_limit() {
        let tracker = RecursionTracker::new(2);

        let _guard1 = tracker.enter().unwrap();
        let _guard2 = tracker.enter().unwrap();
        let result = tracker.enter();

        assert!(result.is_err());
    }

    #[test]
    fn test_timeout_guard_remaining_time() {
        let guard = TimeoutGuard::new(10, "test");
        let remaining = guard.remaining();
        assert!(remaining.as_secs() <= 10);
        assert!(remaining.as_secs() >= 9); // Allow some tolerance
    }
}
