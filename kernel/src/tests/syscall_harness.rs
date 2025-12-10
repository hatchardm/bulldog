// File: kernel/src/tests/syscall_harness.rs
//! Minimal syscall harness for Bulldog
//! Exercises both happy and error paths for syscalls.

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
    // --- sys_write happy path ---
    let msg = b"Hello from harness!\0";
    let ret = unsafe { syscall(SYS_WRITE, 1, msg.as_ptr() as u64, msg.len() as u32) };
    info!("[HARNESS] sys_write returned: {}", ret);
    assert_eq!(ret, 0);

    // --- sys_write bogus pointer (error path) ---
    let bogus_ptr: u64 = 0xFFFF_FFFF_FFFF_FFFF;
    let ret = unsafe { syscall(SYS_WRITE, 1, bogus_ptr, 8) };
    info!("[HARNESS] sys_write bogus ptr returned: {}", ret);
    assert_eq!(ret, u64::MAX);

    // --- sys_open happy path ---
    let path = b"foo.txt\0";
    let fd = unsafe { syscall(SYS_OPEN, path.as_ptr() as u64, 0, 0) };
    info!("[HARNESS] sys_open returned fd: {}", fd);
    assert_eq!(fd, 42);

   // --- sys_open bogus pointer (error path) ---
   // Use a clearly invalid pointer that the guard rejects.
   let bogus_ptr: u64 = 0xFFFF_FFFF_FFFF_FFFF; // non-canonical sentinel
   let fd = unsafe { syscall(SYS_OPEN, bogus_ptr, 0, 0) };
   info!("[HARNESS] sys_open bogus ptr returned fd: {}", fd);
   assert_eq!(fd, u64::MAX);


    // --- sys_exit ---
    let code = unsafe { syscall(SYS_EXIT, 123, 0, 0) };
    info!("[HARNESS] sys_exit returned: {}", code);
    assert_eq!(code, 0);
}




