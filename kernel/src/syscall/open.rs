// File: kernel/src/syscall/open.rs

use crate::syscall::errno::{Errno, err, err_from, errno, strerror};
use crate::syscall::stubs::copy_cstr_from_user;
use crate::syscall::fd::{fd_alloc, FdEntry};
use crate::syscall::stubs::Stdout; // temporary placeholder FileLike
use log::{info, error};
use alloc::boxed::Box;

pub fn sys_open(path_ptr: u64, flags: u64, _mode: u64) -> u64 {
    if path_ptr == 0 {
        return err(errno::ENOENT);
    }

    let mut scratch = [0u8; 256];

    let path = match copy_cstr_from_user(path_ptr, &mut scratch) {
        Ok(p) => p,
        Err(_) => return err(errno::EFAULT),
    };

    if path.is_empty() {
        return err(errno::EINVAL);
    }

    if flags == 0xFFFF_FFFF {
        return err(errno::EINVAL);
    }

    // TEMPORARY: always return a Stdout-like file
    let entry = FdEntry {
        file: Box::new(Stdout),
        flags,
        offset: 0,
    };

    match fd_alloc(entry) {
        Ok(fd) => {
            info!("[OPEN] path=\"{}\" flags={} → fd={}", path, flags, fd);
            fd
        }
        Err(e) => {
            error!(
                "[OPEN] failed to allocate FD → {:?} ({})",
                e,
                strerror(e.num())
            );
            err_from(e)
        }
    }
}






