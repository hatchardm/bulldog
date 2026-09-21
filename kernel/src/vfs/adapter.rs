// File: kernel/src/vfs/adapter.rs

use alloc::boxed::Box;
use alloc::sync::Arc;
use spin::Mutex;

use crate::syscall::errno::Errno;
use crate::syscall::filelike::FileLike;
use crate::vfs::file::FileOps;

pub struct VfsFileLike {
    inner: Arc<Mutex<Box<dyn FileOps>>>,
}

impl VfsFileLike {
    pub fn new(inner: Arc<Mutex<Box<dyn FileOps>>>) -> Self {
        Self { inner }
    }
}

impl FileLike for VfsFileLike {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Errno> {
        self.inner.lock().read(buf)
    }

    fn write(&mut self, buf: &[u8]) -> Result<usize, Errno> {
        self.inner.lock().write(buf)
    }

    fn close(&mut self) -> Result<(), Errno> {
        self.inner.lock().close()
    }
}