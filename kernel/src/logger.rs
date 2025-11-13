// logger.rs

use log::{Record, Level, Metadata, LevelFilter, set_logger, set_max_level};
use crate::writer::{WRITER, LogLevel};
use core::fmt::Write;

/// The global logger instance for the kernel.
pub struct KernelLogger;

/// Static instance used by the log crate.
static LOGGER: KernelLogger = KernelLogger;

impl log::Log for KernelLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Trace // Adjust as needed
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let mut writer = WRITER.lock();
            if let Some(ref mut tw) = *writer {
                let level = match record.level() {
                    Level::Trace => LogLevel::Trace,
                    Level::Debug => LogLevel::Debug,
                    Level::Info  => LogLevel::Info,
                    Level::Warn  => LogLevel::Warn,
                    Level::Error => LogLevel::Error,
                };
                tw.log(level, *record.args()); // zero-allocation, correct type
            }
        }
    }

    fn flush(&self) {}
}

/// Initializes the kernel logger and sets the maximum log level.
pub fn logger_init() {
    set_logger(&LOGGER).unwrap();
    set_max_level(LevelFilter::Trace); // Change to Info, Debug, etc. as needed

    if let Some(ref mut writer) = *WRITER.lock() {
        writer.log(LogLevel::Info, format_args!("Logger initialized"));

    }
}

