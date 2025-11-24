use core::sync::atomic::{AtomicUsize, Ordering};
use log::{self, Level, LevelFilter, Metadata, Record, set_max_level};

use crate::writer::WRITER;

pub static CURRENT_LEVEL: AtomicUsize = AtomicUsize::new(LevelFilter::Info as usize);

struct BulldogLogger;
static LOGGER: BulldogLogger = BulldogLogger;

impl log::Log for BulldogLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        // Defer to the global filter set via set_max_level
        metadata.level() <= log::max_level()
    }

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

    fn flush(&self) {}
}

pub fn logger_init(level: LevelFilter) {
    // Safe here because we are in single-threaded early boot and will only ever set one logger
    unsafe {
        log::set_logger_racy(&LOGGER);
    }
    set_max_level(level);
    CURRENT_LEVEL.store(level as usize, Ordering::Relaxed);

    log::info!("Logger initialized at {:?} level", level);
}




