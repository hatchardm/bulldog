// File: kernel/src/vfs/file.rs
//! Kernel‑internal file abstraction for the future VFS.
//! This does NOT replace FileLike yet — it is additive only.
//! 
use alloc::boxed::Box;
use crate::syscall::errno::Errno;

pub type FileResult<T> = Result<T, Errno>;

/// Unified kernel‑internal file interface.
/// All filesystem backends (ramdisk, devfs, pipes, etc.) will implement this.
pub trait FileOps: Send {
    /// Read into the provided buffer.
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Errno> {
        let _ = buf;
        Err(Errno::ENOSYS)
    }

    /// Write from the provided buffer.
    fn write(&mut self, buf: &[u8]) -> Result<usize, Errno> {
        let _ = buf;
        Err(Errno::ENOSYS)
    }

    /// Close the file.
    fn close(&mut self) -> Result<(), Errno> {
        Err(Errno::ENOSYS)
    }

    /// Clone this file object into a new boxed trait object.
    /// This is required so VfsNode::File can hand out new handles.
    fn clone_box(&self) -> Box<dyn FileOps>;
}

impl Clone for Box<dyn FileOps> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}