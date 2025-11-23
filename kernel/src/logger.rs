use core::sync::atomic::{AtomicU8, Ordering};
use log::{Record, Level, Metadata, LevelFilter, set_logger, set_max_level};
use crate::writer::{WRITER, LogLevel};

/// The global logger instance for the kernel.
pub struct KernelLogger;

/// Static instance used by the log crate.
static LOGGER: KernelLogger = KernelLogger;

/// Atomic to hold the current max level filter (stored as u8).
static CURRENT_LEVEL: AtomicU8 = AtomicU8::new(LevelFilter::Info as u8);

impl log::Log for KernelLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        let current = match CURRENT_LEVEL.load(Ordering::Relaxed) {
            0 => LevelFilter::Off,
            1 => LevelFilter::Error,
            2 => LevelFilter::Warn,
            3 => LevelFilter::Info,
            4 => LevelFilter::Debug,
            5 => LevelFilter::Trace,
            _ => LevelFilter::Info,
        };
        metadata.level() <= current
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
                tw.log(level, record.args().clone());
            }
        }
    }

    fn flush(&self) {}
}

/// Initializes the kernel logger and sets the maximum log level.
pub fn logger_init(level: LevelFilter) {
    set_logger(&LOGGER).unwrap();
    set_max_level(level);
    CURRENT_LEVEL.store(level as u8, Ordering::Relaxed);

    if let Some(ref mut writer) = *WRITER.lock() {
        writer.log(LogLevel::Info, format_args!("Logger initialized at {:?} level", level));
    }
}

/// Allows runtime adjustment of the log level filter.
pub fn set_log_level(level: LevelFilter) {
    set_max_level(level);
    CURRENT_LEVEL.store(level as u8, Ordering::Relaxed);

    if let Some(ref mut writer) = *WRITER.lock() {
        writer.log(LogLevel::Info, format_args!("Log level changed to {:?}", level));
    }
}

