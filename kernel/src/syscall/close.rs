// File: kernel/src/syscall/close.rs
//! sys_close implementation for Bulldog
//! Removes an FD from the current process FD table.

use crate::syscall::errno::{err, errno};
use crate::syscall::fd::current_process_fd_table;
use log::info;

/// Close a file descriptor.
/// Returns:
///   0 on success
///  -EBADF if fd does not exist
pub fn sys_close(fd: u64) -> u64 {
    let mut guard = current_process_fd_table();

    let table = match guard.as_mut() {
        Some(t) => t,
        None => {
            // FD table not initialized — treat as EBADF
            return err(errno::EBADF);
        }
    };

    if fd < 3 {
        // Disallow closing stdin/stdout/stderr for now
        info!("[CLOSE] attempt to close std fd {} → EBADF", fd);
        return err(errno::EBADF);
    }

    if table.remove(&fd).is_some() {
        info!("[CLOSE] closed fd {}", fd);
        0
    } else {
        info!("[CLOSE] unknown fd {} → EBADF", fd);
        err(errno::EBADF)
    }
}


