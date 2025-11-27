use core::sync::atomic::{AtomicUsize, Ordering};
use log::{self, Level, LevelFilter, Metadata, Record, set_max_level};

use crate::writer::WRITER;

/// Tracks the current maximum log level filter.
/// Stored as an atomic so it can be updated safely at runtime.
pub static CURRENT_LEVEL: AtomicUsize = AtomicUsize::new(LevelFilter::Info as usize);

/// Bulldog’s custom logger implementation.
/// Routes log records into the kernel’s framebuffer writer.
struct BulldogLogger;

/// Global logger instance registered with the `log` crate.
static LOGGER: BulldogLogger = BulldogLogger;

impl log::Log for BulldogLogger {
    /// Determines if a log record should be processed based on the global filter.
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= log::max_level()
    }

    /// Handles an incoming log record.
    /// Converts the `log::Level` into Bulldog’s internal `LogLevel` and forwards
    /// the formatted message to the global writer.
    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }
        if let Some(w) = WRITER.lock().as_mut() {
            let lvl = match record.level() {
                Level::Error => crate::writer::LogLevel::Error,
                Level::Warn  => crate::writer::LogLevel::Warn,
                Level::Info  => crate::writer::LogLevel::Info,
                Level::Debug => crate::writer::LogLevel::Debug,
                Level::Trace => crate::writer::LogLevel::Trace,
            };
            w.log(lvl, format_args!("{}", record.args()));
        }
    }

    /// Flush is a no‑op because Bulldog’s writer logs directly to framebuffer.
    fn flush(&self) {}
}

/// Initialize Bulldog’s logger at the given level.
/// - Registers the global `LOGGER` with the `log` crate.
/// - Sets the maximum log level filter.
/// - Stores the level in `CURRENT_LEVEL` for runtime checks.
/// 
/// # Safety
/// This is safe in early boot because only one logger is ever set.
pub fn logger_init(level: LevelFilter) {
    unsafe {
        log::set_logger_racy(&LOGGER);
    }
    set_max_level(level);
    CURRENT_LEVEL.store(level as usize, Ordering::Relaxed);

    log::info!("Logger initialized at {:?} level", level);
}





