// File: kernel/src/syscall/write.rs
//! Minimal sys_write implementation for Bulldog kernel.
//! Validates fd and user buffer, logs the request, and returns success or errno.

use crate::syscall::errno::{err, errno};
use crate::syscall::stubs::{copy_from_user_into, is_user_ptr};
use log::{info, error};

/// sys_write(fd, buf_ptr, len)
/// - Returns EBADF if fd is invalid (0 reserved for stdin).
/// - Returns EFAULT if buf_ptr is invalid.
/// - Otherwise logs the buffer contents and returns 0 (success).
pub fn sys_write(fd: u64, buf_ptr: u64, len: u64) -> u64 {
    // Validate fd: for now, accept only >= 1
    if fd == 0 {
        error!("[WRITE] invalid fd={}", fd);
        return err(errno::EBADF);
    }

    // Validate buffer pointer
    let mut scratch = [0u8; 256];
    match copy_from_user_into(buf_ptr, len as usize, &mut scratch) {
        Ok(buf) => {
            let s = core::str::from_utf8(buf).unwrap_or("<invalid utf8>");
            info!("[WRITE] fd={} buf=\"{}\"", fd, s);
            0 // success
        }
        Err(_) => {
            error!("[WRITE] invalid user buffer {:#x}", buf_ptr);
            err(errno::EFAULT)
        }
    }
}
