// File: kernel/src/syscall/exit.rs
//! sys_exit: clear FDs; then idle, or run a follow-up program.

use core::sync::atomic::{AtomicU8, Ordering};

use crate::syscall::fd::current_process_fd_table;
use log::info;

/// 0 = nothing special (syscall_tests)
/// 1 = go idle after this exit
/// 2 = after this exit, run /bin/goodbye then idle
const AFTER_EXIT_NONE: u8 = 0;
const AFTER_EXIT_IDLE: u8 = 1;
const AFTER_EXIT_CHAIN_GOODBYE: u8 = 2;

static AFTER_EXIT: AtomicU8 = AtomicU8::new(AFTER_EXIT_NONE);

pub fn arm_exit_to_idle() {
    AFTER_EXIT.store(AFTER_EXIT_IDLE, Ordering::SeqCst);
}

pub fn arm_exit_chain_goodbye() {
    AFTER_EXIT.store(AFTER_EXIT_CHAIN_GOODBYE, Ordering::SeqCst);
}

pub fn sys_exit(code: u64) -> u64 {
    info!("[EXIT] process exiting with code={}", code);

    let after = AFTER_EXIT.swap(AFTER_EXIT_NONE, Ordering::SeqCst);

    match after {
        AFTER_EXIT_IDLE => {
            crate::syscall::fd::fd_clear_all();
            info!("[EXIT] FD table cleared");
            crate::memory::unmap_current_user();
            crate::kernel_idle_after_exit();
        }
        #[cfg(feature = "usermode_chain")]
        AFTER_EXIT_CHAIN_GOODBYE => {
            crate::syscall::fd::fd_clear_all();
            info!("[EXIT] FD table cleared");
            crate::memory::unmap_current_user();
            info!("[EXIT] chaining to /bin/goodbye");
            crate::run_usermode_elf("/bin/goodbye", true);
        }
        _ => {
            // syscall_tests: clear entries, keep Option::Some so harness can continue
            crate::syscall::fd::fd_reset_for_harness();
            info!("[EXIT] FD table cleared");
        }
    }

    0
}



/* 

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

*/


