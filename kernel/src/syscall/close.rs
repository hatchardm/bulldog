// File: kernel/src/syscall/close.rs

use crate::syscall::errno::{Errno, err_from};
use crate::syscall::fd::fd_close;
use log::{info, error};

pub fn sys_close(fd: u64) -> u64 {
    match fd_close(fd) {
        Ok(_) => {
            info!("[CLOSE] fd={} → OK", fd);
            0
        }
        Err(e) => {
            error!("[CLOSE] fd={} → {:?} ({})", fd, e, e.num());
            err_from(e)
        }
    }
}



