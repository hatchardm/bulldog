// File: kernel/src/tests/syscall_harness.rs
//! Minimal syscall harness for Bulldog

use crate::syscall::{SYS_WRITE, SYS_EXIT, SYS_OPEN};
use log::info;

/// Inline assembly wrapper to trigger a syscall with up to 3 args.
/// Note: arg2 is u32 so it binds cleanly into edx.
#[inline(always)]
unsafe fn syscall(num: u64, arg0: u64, arg1: u64, arg2: u32) -> u64 {
    let ret: u64;
    core::arch::asm!(
        "int 0x80",
        in("rax") num,       // syscall number
        in("rdi") arg0,      // first argument
        in("rsi") arg1,      // second argument
        in("edx") arg2,      // third argument (32-bit, zero-extends into rdx)
        lateout("rax") ret,  // return value
        options(nostack, preserves_flags)
    );
    ret
}

/// Run a sequence of syscall tests.
pub fn run_syscall_tests() {
    // Test sys_write
    let msg = b"Hello from harness!\0";
    let ret = unsafe { syscall(SYS_WRITE, 1, msg.as_ptr() as u64, msg.len() as u32) };
    info!("[HARNESS] sys_write returned: {}", ret);
    assert_eq!(ret, 0);

    // Test sys_open
    let path = b"foo.txt\0";
    let fd = unsafe { syscall(SYS_OPEN, path.as_ptr() as u64, 0, 0) };
    info!("[HARNESS] sys_open returned fd: {}", fd);
    assert_eq!(fd, 42);

    // Test sys_exit
    let code = unsafe { syscall(SYS_EXIT, 123, 0, 0) };
    info!("[HARNESS] sys_exit returned: {}", code);
    assert_eq!(code, 0);
}


