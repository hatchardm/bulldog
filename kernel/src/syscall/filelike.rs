// File: kernel/src/syscall/filelike.rs

use crate::syscall::errno::Errno;

/// Core polymorphic interface for all file-like objects.
/// Default implementations return EBADF or EINVAL where appropriate.
pub trait FileLike: Send {
    fn read(&mut self, _buf: &mut [u8]) -> Result<usize, Errno> {
        Err(Errno::EBADF)
    }

    fn write(&mut self, _buf: &[u8]) -> Result<usize, Errno> {
        Err(Errno::EBADF)
    }

    fn close(&mut self) -> Result<(), Errno> {
        Ok(())
    }

    fn seek(&mut self, _offset: usize) -> Result<(), Errno> {
        Err(Errno::EINVAL)
    }
}

