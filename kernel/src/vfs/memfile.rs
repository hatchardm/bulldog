// File: kernel/src/vfs/memfile.rs
//! Simple in‑memory file backend for Bulldog's VFS.
//! This is the first concrete FileOps implementation.

use alloc::vec::Vec;
use alloc::boxed::Box;

use crate::syscall::errno::Errno;
use crate::vfs::file::{FileOps, FileResult};

/// A simple in‑memory file.
/// Supports read, write, close, and cloning.
pub struct MemFile {
    data: Vec<u8>,
    offset: usize,
}

impl MemFile {
    /// Create a new MemFile with optional initial contents.
    pub fn new(initial: Vec<u8>) -> Self {
        Self {
            data: initial,
            offset: 0,
        }
    }
}

impl FileOps for MemFile {
    fn read(&mut self, buf: &mut [u8]) -> FileResult<usize> {
        if self.offset >= self.data.len() {
            return Ok(0); // EOF
        }

        let remaining = &self.data[self.offset..];
        let count = remaining.len().min(buf.len());

        buf[..count].copy_from_slice(&remaining[..count]);
        self.offset += count;

        Ok(count)
    }

    fn write(&mut self, buf: &[u8]) -> FileResult<usize> {
        // Ensure capacity
        if self.offset + buf.len() > self.data.len() {
            self.data.resize(self.offset + buf.len(), 0);
        }

        self.data[self.offset..self.offset + buf.len()].copy_from_slice(buf);
        self.offset += buf.len();

        Ok(buf.len())
    }

    fn close(&mut self) -> FileResult<()> {
        Ok(())
    }

    fn clone_box(&self) -> Box<dyn FileOps> {
        Box::new(Self {
            data: self.data.clone(),
            offset: self.offset,
        })
    }
}