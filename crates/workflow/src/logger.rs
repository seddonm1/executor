use crate::bindings::{self, component::workflow::abi::Level as BindingsLevel, GuestToHost};
use log::{Level, LevelFilter, Metadata, Record, SetLoggerError};

/// A simple logger implementation.
static LOGGER: SimpleLogger = SimpleLogger;

/// Represents a simple logger.
struct SimpleLogger;

/// Initializes the logger with the specified log level.
///
/// # Arguments
///
/// * `level` - The maximum log level to be set.
///
/// # Returns
///
/// Returns `Ok(())` if the logger was successfully set, or an error if it failed.
pub fn init(level: LevelFilter) -> Result<(), SetLoggerError> {
    log::set_logger(&LOGGER).map(|()| log::set_max_level(level))
}

impl log::Log for SimpleLogger {
    /// Checks if logging is enabled for the given metadata.
    ///
    /// This implementation always returns true, meaning all log messages are enabled.
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    /// Logs a record with the appropriate level.
    ///
    /// This method translates the log levels from the `log` crate to the corresponding
    /// levels in the `bindings` module and calls the appropriate logging function.
    ///
    /// # Arguments
    ///
    /// * `record` - The log record to be logged.
    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let bindings_level = match record.level() {
                Level::Error => BindingsLevel::Error(record.args().to_string()),
                Level::Warn => BindingsLevel::Warn(record.args().to_string()),
                Level::Info => BindingsLevel::Info(record.args().to_string()),
                Level::Debug => BindingsLevel::Debug(record.args().to_string()),
                Level::Trace => BindingsLevel::Trace(record.args().to_string()),
            };
            bindings::call(&GuestToHost::Log(bindings_level));
        }
    }

    /// Flushes any buffered records.
    ///
    /// This implementation does nothing as there is no buffering.
    fn flush(&self) {}
}
