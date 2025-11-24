use log::{LevelFilter, Log, Metadata, Record};
use std::io::Write;
use std::sync::Mutex;

pub struct RfmtLogger {
    level: LevelFilter,
    output: Mutex<Box<dyn Write + Send>>,
}

impl RfmtLogger {
    pub fn new(level: LevelFilter) -> Self {
        Self {
            level,
            output: Mutex::new(Box::new(std::io::stderr())),
        }
    }

    pub fn with_output(mut self, output: Box<dyn Write + Send>) -> Self {
        self.output = Mutex::new(output);
        self
    }

    pub fn init() {
        let logger = Self::new(LevelFilter::Info);
        log::set_boxed_logger(Box::new(logger)).expect("Failed to initialize logger");
        log::set_max_level(LevelFilter::Trace);
    }

    pub fn init_with_level(level: LevelFilter) {
        let logger = Self::new(level);
        log::set_boxed_logger(Box::new(logger)).expect("Failed to initialize logger");
        log::set_max_level(level);
    }
}

impl Log for RfmtLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.level
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        let mut output = self.output.lock().unwrap();

        writeln!(
            output,
            "[{}] {} - {}",
            record.level(),
            record.target(),
            record.args()
        )
        .ok();
    }

    fn flush(&self) {
        let mut output = self.output.lock().unwrap();
        output.flush().ok();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use log::{debug, error, info, trace, warn};
    use std::sync::{Arc, Mutex};

    struct TestWriter {
        data: Arc<Mutex<Vec<u8>>>,
    }

    impl TestWriter {
        fn new() -> (Self, Arc<Mutex<Vec<u8>>>) {
            let data = Arc::new(Mutex::new(Vec::new()));
            (
                Self {
                    data: Arc::clone(&data),
                },
                data,
            )
        }
    }

    impl Write for TestWriter {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            self.data.lock().unwrap().write(buf)
        }

        fn flush(&mut self) -> std::io::Result<()> {
            self.data.lock().unwrap().flush()
        }
    }

    #[test]
    fn test_logger_creation() {
        let logger = RfmtLogger::new(LevelFilter::Info);
        assert!(logger.enabled(&Metadata::builder().level(log::Level::Info).build()));
        assert!(!logger.enabled(&Metadata::builder().level(log::Level::Debug).build()));
    }

    #[test]
    fn test_logger_level_filtering() {
        let logger = RfmtLogger::new(LevelFilter::Warn);

        assert!(logger.enabled(&Metadata::builder().level(log::Level::Error).build()));
        assert!(logger.enabled(&Metadata::builder().level(log::Level::Warn).build()));
        assert!(!logger.enabled(&Metadata::builder().level(log::Level::Info).build()));
        assert!(!logger.enabled(&Metadata::builder().level(log::Level::Debug).build()));
        assert!(!logger.enabled(&Metadata::builder().level(log::Level::Trace).build()));
    }

    #[test]
    fn test_logger_output() {
        let (writer, data) = TestWriter::new();
        let logger = RfmtLogger::new(LevelFilter::Info).with_output(Box::new(writer));

        let record = Record::builder()
            .level(log::Level::Info)
            .target("test")
            .args(format_args!("test message"))
            .build();

        logger.log(&record);
        logger.flush();

        let output = String::from_utf8(data.lock().unwrap().clone()).unwrap();
        assert!(output.contains("[INFO]"));
        assert!(output.contains("test"));
        assert!(output.contains("test message"));
    }
}
