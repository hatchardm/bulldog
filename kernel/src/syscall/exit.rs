// File: kernel/src/syscall/exit.rs
//! Expanded sys_exit implementation for Bulldog kernel.
//! Logs the exit code and always returns success (stub).

use log::info;

/// sys_exit(code, _, _)
/// - Logs the exit code.
/// - Always returns 0 (success).
/// - In a real kernel, this would tear down the process and never return.
pub fn sys_exit(code: u64, _unused1: u64, _unused2: u64) -> u64 {
    info!("[EXIT] process exited with code {}", code);
    0
}


