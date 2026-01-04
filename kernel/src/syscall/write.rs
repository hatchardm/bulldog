// File: kernel/src/syscall/write.rs

use crate::syscall::errno::{Errno, err_from};
use crate::syscall::fd::fd_get;
use crate::syscall::stubs::copy_from_user;
use log::{info, error};

pub fn sys_write(fd: u64, buf_ptr: u64, len: u64) -> u64 {
    // Zero-length write is trivially successful
    if len == 0 {
        return 0;
    }

    // Resolve the file descriptor
    let entry = match fd_get(fd) {
        Ok(e) => e,
        Err(e) => return err_from(e),
    };

    let len = len as usize;

    // Temporary fixed-size buffer until heap is online
    const MAX_WRITE: usize = 4096;
    let mut buf = [0u8; MAX_WRITE];
    let scratch = &mut buf[..len.min(MAX_WRITE)];

    // Copy user → kernel
    if let Err(_) = copy_from_user(buf_ptr, scratch) {
        return err_from(Errno::EFAULT);
    }

    // Perform the write
    match entry.file.write(scratch) {
        Ok(n) => n as u64,
        Err(e) => err_from(e),
    }
}








