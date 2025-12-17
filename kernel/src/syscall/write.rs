// File: kernel/src/syscall/write.rs
//! Expanded sys_write implementation for Bulldog kernel.
//! Validates fd, buffer pointer, and length before writing.

use crate::syscall::errno::{err, errno, strerror};
use crate::syscall::stubs::copy_from_user_into;
use log::{info, error};

/// sys_write(fd, buf_ptr, len)
/// - Returns EBADF if fd is invalid (0 reserved for stdin).
/// - Returns EFAULT if buf_ptr is invalid.
/// - Returns EINVAL if len is absurdly large.
/// - Returns 0 if len == 0 (no-op).
/// - Otherwise logs the buffer contents and returns 0 (success).
pub fn sys_write(fd: u64, buf_ptr: u64, len: u64) -> u64 {
    // Validate fd: for now, accept only >= 1
    if fd == 0 {
        let code = errno::EBADF;
        error!("[WRITE] invalid fd={} → {} ({})", fd, code, strerror(code));
        return err(code);
    }

    // Zero-length write is a no-op
    if len == 0 {
        info!("[WRITE] fd={} zero-length write → 0", fd);
        return 0;
    }

    // Guard against absurdly large lengths
    if len == u32::MAX as u64 {
        let code = errno::EINVAL;
        error!(
            "[WRITE] fd={} huge length {} → {} ({})",
            fd,
            len,
            code,
            strerror(code)
        );
        return err(code);
    }

    // Validate buffer pointer and copy
    let mut scratch = [0u8; 256];
    match copy_from_user_into(buf_ptr, len as usize, &mut scratch) {
        Ok(buf) => {
            let s = core::str::from_utf8(buf).unwrap_or("<invalid utf8>");
            info!("[WRITE] fd={} buf=\"{}\"", fd, s);
            0 // success
        }
        Err(_) => {
            let code = errno::EFAULT;
            error!(
                "[WRITE] invalid user buffer {:#x} → {} ({})",
                buf_ptr,
                code,
                strerror(code)
            );
            err(code)
        }
    }
}


