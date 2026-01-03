// File: kernel/src/syscall/open.rs
//! Expanded sys_open implementation for Bulldog kernel.
//! Validates user path pointers, logs the request, and hands out FDs via the FD table.

use crate::syscall::errno::{err, errno, strerror};
use crate::syscall::stubs::copy_cstr_from_user;
use crate::syscall::fd::{current_process_fd_table, Stdout};
use log::{info, error};
use alloc::boxed::Box;

/// Maximum file descriptor number handed out by sys_open.
/// FDs 0,1,2 are reserved for stdin/stdout/stderr, so usable range is [3, MAX_FD].
const MAX_FD: u64 = 64; // adjust as needed as Bulldog grows

/// sys_open(path_ptr, flags, mode)
/// - Returns ENOENT if path pointer is null.
/// - Returns EFAULT if path pointer is invalid or not a valid C string.
/// - Returns EINVAL if path is empty or flags unsupported.
/// - Returns EMFILE if no more FDs are available.
/// - Otherwise logs the path and flags, inserts a FileLike object, and returns the fd.
pub fn sys_open(path_ptr: u64, flags: u64, _mode: u64) -> u64 {
    if path_ptr == 0 {
        let code = errno::ENOENT;
        error!("[OPEN] null path pointer → {} ({})", code, strerror(code));
        return err(code);
    }

    let mut scratch = [0u8; 256];
    match copy_cstr_from_user(path_ptr, &mut scratch) {
        Ok(path) => {
            if path.is_empty() {
                let code = errno::EINVAL;
                error!("[OPEN] empty path → {} ({})", code, strerror(code));
                return err(code);
            }
            if flags == 0xFFFF_FFFF {
                let code = errno::EINVAL;
                error!(
                    "[OPEN] unsupported flags {:#x} → {} ({})",
                    flags, code, strerror(code)
                );
                return err(code);
            }

            // Lock the FD table and insert a new Stdout object.
            let mut guard = current_process_fd_table();
            let table = guard.as_mut().expect("FD table not initialized");

            // Find the lowest available fd in [3, MAX_FD].
            let mut fd = 3u64;
            while fd <= MAX_FD {
                if !table.contains_key(&fd) {
                    break;
                }
                fd += 1;
            }

            if fd > MAX_FD {
                let code = errno::EMFILE;
                error!(
                    "[OPEN] FD table exhausted (MAX_FD={}) → {} ({})",
                    MAX_FD,
                    code,
                    strerror(code)
                );
                return err(code);
            }

            table.insert(fd, Box::new(Stdout));
            info!("[OPEN] path=\"{}\" flags={} → fd={}", path, flags, fd);
            fd
        }
        Err(_) => {
            let code = errno::EFAULT;
            error!(
                "[OPEN] invalid user path ptr {:#x} → {} ({})",
                path_ptr, code, strerror(code)
            );
            err(code)
        }
    }
}





