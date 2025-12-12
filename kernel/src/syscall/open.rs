// File: kernel/src/syscall/open.rs
//! Minimal sys_open implementation for Bulldog kernel.
//! Validates user path pointers, logs the request, and hands out incrementing FDs.

use crate::syscall::errno::{err, errno};
use crate::syscall::stubs::copy_cstr_from_user;
use core::sync::atomic::{AtomicU64, Ordering};
use log::{info, error};

/// Global file descriptor counter.
/// Starts at 3 (0,1,2 reserved for stdin/stdout/stderr).
static NEXT_FD: AtomicU64 = AtomicU64::new(3);

/// sys_open(path_ptr, flags, mode)
/// - Returns ENOENT if path pointer is null.
/// - Returns EFAULT if path pointer is invalid or not a valid C string.
/// - Otherwise logs the path and flags, and hands out incrementing FDs.
pub fn sys_open(path_ptr: u64, flags: u64, _mode: u64) -> u64 {
    if path_ptr == 0 {
        error!("[OPEN] null path pointer");
        return err(errno::ENOENT);
    }

    let mut scratch = [0u8; 256];
    match copy_cstr_from_user(path_ptr, &mut scratch) {
        Ok(path) => {
            info!("[OPEN] path=\"{}\" flags={}", path, flags);
            NEXT_FD.fetch_add(1, Ordering::SeqCst)
        }
        Err(_) => {
            error!("[OPEN] invalid user path ptr {:#x}", path_ptr);
            err(errno::EFAULT)
        }
    }
}

