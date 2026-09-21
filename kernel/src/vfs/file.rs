// File: kernel/src/vfs/file.rs

use alloc::boxed::Box;
use crate::syscall::errno::Errno;

pub type FileResult<T> = Result<T, Errno>;

/// Unified kernel‑internal file interface.
pub trait FileOps: Send {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Errno> {
        let _ = buf;
        Err(Errno::ENOSYS)
    }

    fn write(&mut self, buf: &[u8]) -> Result<usize, Errno> {
        let _ = buf;
        Err(Errno::ENOSYS)
    }

    fn close(&mut self) -> Result<(), Errno> {
        Err(Errno::ENOSYS)
    }

    /// Reset internal offset to the beginning.
    fn rewind(&mut self) {
        // default: do nothing
    }

    fn clone_box(&self) -> Box<dyn FileOps>;
}

impl Clone for Box<dyn FileOps> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}