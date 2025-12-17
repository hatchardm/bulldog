// kernel/src/logger.rs
#![allow(dead_code)]

use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use core::fmt::Write as FmtWrite;
use log::{self, Level, LevelFilter, Metadata, Record, set_max_level};
use crate::writer::WRITER;
use crate::serial::serial_print;

/// Tracks the current maximum log level filter.
pub static CURRENT_LEVEL: AtomicUsize = AtomicUsize::new(LevelFilter::Info as usize);

/// Framebuffer readiness flag — set to true once WRITER is fully initialized.
static FB_READY: AtomicBool = AtomicBool::new(false);

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

/// Composite logger: serial (optional, heapless) + framebuffer (guarded).
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

        // 1) Serial backend (heapless). Controlled by feature flag.
        #[cfg(feature = "serial_log")]
        {
            let mut sw = SerialWriter;
            let _ = write!(&mut sw, "[{}] ", record.level());
            let _ = sw.write_fmt(*record.args());
            serial_print("\n");
        }

        // 2) Framebuffer backend (only if WRITER is ready).
        if FB_READY.load(Ordering::Relaxed) {
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
    }

    fn flush(&self) {}
}

/// Initialize the global logger and set the filter.
/// Call `set_framebuffer_ready(true)` after WRITER is initialized.
pub fn logger_init(level: LevelFilter) {
    unsafe {
        log::set_logger_racy(&LOGGER);
    }
    set_max_level(level);
    CURRENT_LEVEL.store(level as usize, Ordering::Relaxed);

    // Safe: serial is heapless; framebuffer is guarded until ready.
    log::info!("Logger initialized at {:?} level", level);
}





