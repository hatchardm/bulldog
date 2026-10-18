// File: kernel/src/syscall/exit.rs
//! sys_exit: clear FDs; optionally never return to user (idle after real usermode exit).

use core::sync::atomic::{AtomicBool, Ordering};

use crate::syscall::fd::current_process_fd_table;
use log::info;

static EXIT_TO_IDLE: AtomicBool = AtomicBool::new(false);

/// Call once before `enter_user_mode` (usermode_hello path).
pub fn arm_exit_to_idle() {
    EXIT_TO_IDLE.store(true, Ordering::SeqCst);
}

pub fn sys_exit(code: u64) -> u64 {
    info!("[EXIT] process exiting with code={}", code);

    let mut guard = current_process_fd_table();
    if let Some(table) = guard.as_mut() {
        table.clear();
        info!("[EXIT] FD table cleared");
    } else {
        info!("[EXIT] FD table not initialized");
    }

    if EXIT_TO_IDLE.load(Ordering::SeqCst) {
        EXIT_TO_IDLE.store(false, Ordering::SeqCst);
        crate::kernel_idle_after_exit(); // -> !  never returns
    }

    0 // syscall_tests harness continues
}




