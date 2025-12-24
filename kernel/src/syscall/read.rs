// File: kernel/src/syscall/read.rs
//! Stubbed sys_read implementation for Bulldog kernel.
//! Validates fd and buffer pointer, uses FD table when present.

use crate::syscall::errno::{err, errno, strerror};
use crate::syscall::stubs::is_user_ptr;
use crate::syscall::fd::current_process_fd_table;
use log::{info, error};

/// sys_read(fd, buf_ptr, len)
/// - Returns EBADF if fd invalid.
/// - Returns EFAULT if buf_ptr invalid.
/// - Returns 0 if len == 0.
/// - Otherwise calls FileLike::read and returns bytes read.
pub fn sys_read(fd: u64, buf_ptr: u64, len: u64) -> u64 {
    if len == 0 {
        info!("[READ] fd={} zero-length read → 0", fd);
        return 0;
    }

    if !is_user_ptr(buf_ptr) {
        let code = errno::EFAULT;
        error!("[READ] invalid user buffer {:#x} → {} ({})", buf_ptr, code, strerror(code));
        return err(code);
    }

    let mut guard = current_process_fd_table();
    match guard.as_mut() {
        Some(table) => {
            if let Some(obj) = table.get_mut(&fd) {
                let mut scratch = [0u8; 256];
                let slice_len = core::cmp::min(len as usize, scratch.len());
                let n = obj.read(&mut scratch[..slice_len]);

                info!("[READ] fd={} read={} bytes", fd, n);
                n as u64
            } else {
                let code = errno::EBADF;
                error!("[READ] unknown fd={} → {} ({})", fd, code, strerror(code));
                err(code)
            }
        }
        None => {
            let code = errno::EBADF;
            error!("[READ] FD table not initialized → {} ({})", code, strerror(code));
            err(code)
        }
    }
}
