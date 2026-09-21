// kernel/src/logger.rs
#![allow(dead_code)]

use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use core::fmt::Write as FmtWrite;
use core::fmt::Write;
use log::{self, LevelFilter, Metadata, Record, set_max_level};
use crate::serial::serial_print;
use crate::writer::WRITER;
use alloc::format;

/// Tracks the current maximum log level filter.
pub static CURRENT_LEVEL: AtomicUsize = AtomicUsize::new(LevelFilter::Info as usize);

/// Framebuffer readiness flag — set to true once WRITER is initialized.
pub static FB_READY: AtomicBool = AtomicBool::new(false);

/// Mark framebuffer writer readiness.
pub fn set_framebuffer_ready(ready: bool) {
    FB_READY.store(ready, Ordering::Relaxed);
}

/// Heapless serial writer: forwards fmt writes to COM1 without allocation.
struct SerialWriter;

impl FmtWrite for SerialWriter {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        serial_print(s);
        Ok(())
    }
}

/// Composite logger: serial + framebuffer (when ready).
struct CompositeLogger;

static LOGGER: CompositeLogger = CompositeLogger;

impl log::Log for CompositeLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= log::max_level()
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        // --- Serial backend (always enabled) ---
        let mut sw = SerialWriter;
        let _ = write!(&mut sw, "[{}] ", record.level());
        let _ = sw.write_fmt(*record.args());
        serial_print("\n");

        // --- Framebuffer backend (only if WRITER is ready) ---
        // --- Framebuffer backend (only if WRITER is ready) ---
if FB_READY.load(Ordering::Relaxed) {
    let mut guard = WRITER.lock();
    if let Some(w) = guard.as_mut() {

        // Map log::Level → LogLevel
        let lvl = match record.level() {
            log::Level::Error => crate::writer::LogLevel::Error,
            log::Level::Warn  => crate::writer::LogLevel::Warn,
            log::Level::Info  => crate::writer::LogLevel::Info,
            log::Level::Debug => crate::writer::LogLevel::Debug,
            log::Level::Trace => crate::writer::LogLevel::Trace,
        };

        // Set color based on log level
        w.set_log_level_color(lvl);

        // Write prefix
        w.write_str("[");
        w.write_str(record.level().as_str());
        w.write_str("] ");

        // Write message (heapless)
        use core::fmt::Write;
        let _ = write!(w, "{}\n", record.args());
    }
}

    }

    fn flush(&self) {}
}

/// Initialize the global logger and set the filter.
pub fn logger_init(level: LevelFilter) {
    unsafe {
        log::set_logger_racy(&LOGGER);
    }
    set_max_level(level);
    CURRENT_LEVEL.store(level as usize, Ordering::Relaxed);

    log::info!("Logger initialized at {:?} level", level);
}




