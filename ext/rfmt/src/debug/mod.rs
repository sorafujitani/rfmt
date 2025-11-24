pub mod macros;

use log::{debug, trace};
use std::time::Instant;

pub struct DebugContext {
    phase: String,
    start_time: Instant,
    checkpoints: Vec<(String, Instant)>,
}

impl DebugContext {
    pub fn new(phase: impl Into<String>) -> Self {
        let phase = phase.into();
        debug!("Starting phase: {}", phase);

        Self {
            phase,
            start_time: Instant::now(),
            checkpoints: Vec::new(),
        }
    }

    pub fn checkpoint(&mut self, name: impl Into<String>) {
        let name = name.into();
        let now = Instant::now();

        debug!(
            "[{}] Checkpoint '{}' at {:?}",
            self.phase,
            name,
            now.duration_since(self.start_time)
        );

        self.checkpoints.push((name, now));
    }

    pub fn complete(self) {
        let total_time = self.start_time.elapsed();

        debug!("Completed phase '{}' in {:?}", self.phase, total_time);

        // チェックポイント間の時間を表示
        if self.checkpoints.len() >= 2 {
            for window in self.checkpoints.windows(2) {
                let (prev_name, prev_time) = &window[0];
                let (curr_name, curr_time) = &window[1];
                let duration = curr_time.duration_since(*prev_time);

                trace!("  {} -> {} : {:?}", prev_name, curr_name, duration);
            }
        }
    }

    pub fn phase(&self) -> &str {
        &self.phase
    }

    pub fn elapsed(&self) -> std::time::Duration {
        self.start_time.elapsed()
    }

    pub fn checkpoint_count(&self) -> usize {
        self.checkpoints.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_debug_context_creation() {
        let ctx = DebugContext::new("test_phase");
        assert_eq!(ctx.phase(), "test_phase");
        assert_eq!(ctx.checkpoint_count(), 0);
    }

    #[test]
    fn test_checkpoint() {
        let mut ctx = DebugContext::new("test");

        ctx.checkpoint("first");
        assert_eq!(ctx.checkpoint_count(), 1);

        ctx.checkpoint("second");
        assert_eq!(ctx.checkpoint_count(), 2);
    }

    #[test]
    fn test_elapsed_time() {
        let ctx = DebugContext::new("timing_test");

        thread::sleep(Duration::from_millis(10));

        let elapsed = ctx.elapsed();
        assert!(elapsed >= Duration::from_millis(10));
    }

    #[test]
    fn test_complete() {
        let mut ctx = DebugContext::new("complete_test");

        ctx.checkpoint("start");
        thread::sleep(Duration::from_millis(5));
        ctx.checkpoint("middle");
        thread::sleep(Duration::from_millis(5));
        ctx.checkpoint("end");

        // complete() should not panic
        ctx.complete();
    }

    #[test]
    fn test_multiple_phases() {
        let ctx1 = DebugContext::new("phase1");
        let ctx2 = DebugContext::new("phase2");

        assert_eq!(ctx1.phase(), "phase1");
        assert_eq!(ctx2.phase(), "phase2");
    }
}
