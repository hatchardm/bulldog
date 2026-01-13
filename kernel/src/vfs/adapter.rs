use alloc::boxed::Box;
use crate::syscall::errno::Errno;
use crate::syscall::filelike::FileLike;
use crate::vfs::file::FileOps;

/// Temporary adapter so FileOps can be used as FileLike.
pub struct VfsFileLike {
    inner: Box<dyn FileOps>,
}

impl VfsFileLike {
    pub fn new(inner: Box<dyn FileOps>) -> Self {
        Self { inner }
    }
}

impl FileLike for VfsFileLike {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Errno> {
        self.inner.read(buf)
    }

    fn write(&mut self, buf: &[u8]) -> Result<usize, Errno> {
        self.inner.write(buf)
    }

    fn close(&mut self) -> Result<(), Errno> {
        self.inner.close()
    }
}