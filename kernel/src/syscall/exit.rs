// File: kernel/src/syscall/exit.rs
//! Stubbed sys_exit implementation for Bulldog kernel.
//! Logs the exit code and clears the FD table.

use crate::syscall::fd::current_process_fd_table;
use log::info;

pub fn sys_exit(code: u64) -> u64 {
    info!("[EXIT] process exiting with code={}", code);

    let mut guard = current_process_fd_table();
    if let Some(table) = guard.as_mut() {
        table.clear();
        info!("[EXIT] FD table cleared");
    } else {
        info!("[EXIT] FD table not initialized");
    }

    0
}






