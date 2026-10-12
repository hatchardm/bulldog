use alloc::boxed::Box;
use alloc::vec::Vec;

use crate::syscall::errno::Errno;
use crate::syscall::fd::{current_process_fd_table, fd_alloc, fd_get, fd_close};
use crate::vfs::resolve::resolve_path;
use crate::vfs::adapter::VfsFileLike;
use crate::syscall::filelike::FileLike;

pub mod file;
pub mod node;
pub mod mount;
pub mod resolve;
pub mod memfile;
pub mod ops;
pub mod adapter;
pub mod init;

// -------------------------
// Public VFS API
// -------------------------

pub fn open(path: &str, _flags: u64) -> Result<u64, Errno> {
    let handle = resolve_path(path)?;
    let filelike = VfsFileLike::new(handle);

    let entry = crate::syscall::fd::FdEntry {
        file: Box::new(filelike),
        flags: 0,
        offset: 0,
    };

    fd_alloc(entry)
}

pub fn read(fd: u64, buf: &mut [u8]) -> Result<usize, Errno> {
    let file = fd_get(fd)?;
    file.file.read(buf)
}

pub fn close(fd: u64) -> Result<(), Errno> {
    fd_close(fd)
}

// -------------------------
// Convenience helper
// -------------------------

pub fn read_all(path: &str) -> Result<Vec<u8>, ()> {
    let fd = open(path, 0).map_err(|_| ())?;
    let mut buf = Vec::new();
    let mut tmp = [0u8; 4096];

    loop {
        let n = read(fd, &mut tmp).map_err(|_| ())?;
        if n == 0 {
            break;
        }
        buf.extend_from_slice(&tmp[..n]);
    }

    close(fd).map_err(|_| ())?;
    Ok(buf)
}
