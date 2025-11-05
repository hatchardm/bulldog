// logger.rs
use log::{Record, Level, Metadata, LevelFilter};
use core::fmt::Write;
use crate::writer::WRITER;

pub struct KernelLogger;

impl log::Log for KernelLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info // tweakable
    }

    fn log(&self, record: &Record) {
    if self.enabled(record.metadata()) {
        let mut writer = WRITER.lock();
        if let Some(ref mut tw) = *writer {
            let _ = write!(tw, "[{}] {}\n", record.level(), record.args());
        }
    }
}


    fn flush(&self) {}
}
