// kernel/src/logger.rs
#![allow(dead_code)]

use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use core::fmt::Write as FmtWrite;
use log::{self, LevelFilter, Metadata, Record, set_max_level};
use crate::serial::serial_print;

/// Tracks the current maximum log level filter.
pub static CURRENT_LEVEL: AtomicUsize = AtomicUsize::new(LevelFilter::Info as usize);

/// Framebuffer readiness flag — currently unused but kept for future expansion.
static FB_READY: AtomicBool = AtomicBool::new(false);

pub fn set_framebuffer_ready(_ready: bool) {
    // Disabled for now — framebuffer logging not active
}

/// Heapless serial writer: forwards fmt writes to COM1 without allocation.
struct SerialWriter;

impl FmtWrite for SerialWriter {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        serial_print(s);
        Ok(())
    }
}

/// Composite logger: currently serial-only.
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

        // Serial backend (always enabled)
        let mut sw = SerialWriter;
        let _ = write!(&mut sw, "[{}] ", record.level());
        let _ = sw.write_fmt(*record.args());
        serial_print("\n");
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



