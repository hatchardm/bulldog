// File: kernel/src/syscall/read.rs

use crate::syscall::errno::{Errno, err_from};
use crate::syscall::fd::fd_get;
use crate::syscall::stubs::copy_from_user;
use log::{info, error};

pub fn sys_read(fd: u64, buf_ptr: u64, len: u64) -> u64 {
    if len == 0 {
        return 0;
    }

    let entry = match fd_get(fd) {
        Ok(e) => e,
        Err(e) => return err_from(e),
    };

    let len = len as usize;

    const MAX_READ: usize = 4096;
    let mut buf = [0u8; MAX_READ];
    let scratch = &mut buf[..len.min(MAX_READ)];

    // Perform the read: device → kernel
    let n = match entry.file.read(scratch) {
        Ok(n) => n,
        Err(e) => return err_from(e),
    };

    // Copy kernel → user
    if let Err(_) = crate::syscall::stubs::copy_to_user(buf_ptr, &scratch[..n]) {
        return err_from(Errno::EFAULT);
    }

    n as u64
}


