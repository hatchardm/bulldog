// File: kernel/src/syscall/write.rs

use crate::syscall::fd::current_process_fd_table;
use core::slice;
use log::{info, error};

const EBADF: u64  = 9;
const EFAULT: u64 = 14;
const EINVAL: u64 = 22;
const MAX_WRITE: usize = 4096;
const HIGHER_HALF_BASE: u64 = 0x0000_0080_0000_0000;

/// Syscall entry point: write(fd, buf_ptr, len)
pub fn sys_write(fd: u64, buf_ptr: u64, len_u64: u64) -> u64 {
    if len_u64 == 0 {
        info!("[WRITE] fd={} zero-length write → 0", fd);
        return 0;
    }

    let len: usize = match usize::try_from(len_u64) {
        Ok(l) => l,
        Err(_) => {
            error!("[WRITE] length {} does not fit usize → EINVAL", len_u64);
            return -(EINVAL as i64) as u64;
        }
    };

    if len > MAX_WRITE {
        error!("[WRITE] length {} exceeds MAX_WRITE={} → EINVAL", len, MAX_WRITE);
        return -(EINVAL as i64) as u64;
    }

    // IMPORTANT: don't classify pointer overflow as EINVAL.
    // Let copy_from_user decide and return EFAULT on invalid/overflow.
    // (Or if you want an early check, map overflow to EFAULT.)
    if buf_ptr.checked_add(len as u64).is_none() {
        error!("[WRITE] pointer overflow buf_ptr=0x{:x} len={} → EFAULT", buf_ptr, len);
        return -(EFAULT as i64) as u64;
    }

    let mut guard = current_process_fd_table();
    let table = match guard.as_mut() {
        Some(t) => t,
        None => {
            error!("[WRITE] FD table not initialized → EBADF");
            return -(EBADF as i64) as u64;
        }
    };

    if fd == 0 {
        error!("[WRITE] attempt to write to stdin fd=0 → EBADF");
        return -(EBADF as i64) as u64;
    }

    let file = match table.get_mut(&fd) {
        Some(f) => f,
        None => {
            error!("[WRITE] unknown fd={} → EBADF", fd);
            return -(EBADF as i64) as u64;
        }
    };

    let slice: &'static [u8] = match copy_from_user(buf_ptr, len) {
        Some(s) => s,
        None => {
            error!("[WRITE] invalid user buffer 0x{:016x} → EFAULT", buf_ptr);
            return -(EFAULT as i64) as u64;
        }
    };

    let wrote = file.write(slice);
    info!("[WRITE] fd={} wrote={} bytes", fd, wrote);
    wrote as u64
}

/// Harness-only stub: accept higher-half pointers and construct a slice.
fn copy_from_user(ptr: u64, len: usize) -> Option<&'static [u8]> {
    if len == 0 {
        return Some(&[]);
    }
    if ptr == 0 {
        return None;
    }
    // Reject non-higher-half and overflow; classify as EFAULT at call site.
    if ptr < HIGHER_HALF_BASE || ptr.checked_add(len as u64).is_none() {
        return None;
    }
    // SAFETY: Harness only. Assume buffer is mapped and readable.
    unsafe { Some(slice::from_raw_parts(ptr as *const u8, len)) }
}







