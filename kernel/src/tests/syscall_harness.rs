// File: kernel/src/tests/syscall_harness.rs
//! Expanded syscall harness for Bulldog
//! Exercises happy paths and richer error paths for syscalls.

use crate::syscall::{SYS_WRITE, SYS_EXIT, SYS_OPEN};
use log::info;
use crate::syscall::errno::{err, errno};

/// Inline assembly wrapper to trigger a syscall with up to 3 args.
#[inline(always)]
unsafe fn syscall(num: u64, arg0: u64, arg1: u64, arg2: u32) -> u64 {
    let ret: u64;
    core::arch::asm!(
        "int 0x80",
        in("rax") num,
        in("rdi") arg0,
        in("rsi") arg1,
        in("edx") arg2,
        lateout("rax") ret,
        options(nostack, preserves_flags)
    );
    ret
}

// --- typed wrappers for clarity ---
fn sys_write(fd: u64, buf: *const u8, len: u32) -> u64 {
    unsafe { syscall(SYS_WRITE, fd, buf as u64, len) }
}

fn sys_open(path: *const u8, flags: u64) -> u64 {
    unsafe { syscall(SYS_OPEN, path as u64, flags, 0) }
}

fn sys_exit(code: u64) -> u64 {
    unsafe { syscall(SYS_EXIT, code, 0, 0) }
}

/// Run a sequence of syscall tests.
pub fn run_syscall_tests() {
    // --- sys_write happy path ---
    let msg = b"Hello from harness!\0";
    let ret = sys_write(1, msg.as_ptr(), msg.len() as u32);
    info!("[HARNESS] sys_write returned: {}", ret);
    assert_eq!(ret, 0);

    // --- sys_write bogus pointer ---
    let bogus_ptr: u64 = 0xFFFF_FFFF_FFFF_FFFF;
    let ret = unsafe { syscall(SYS_WRITE, 1, bogus_ptr, 8) };
    info!("[HARNESS] sys_write bogus ptr returned: {}", ret);
    assert_eq!(ret, err(errno::EFAULT));

    // --- sys_write invalid fd ---
    let msg = b"Hello\0";
    let ret = sys_write(0, msg.as_ptr(), msg.len() as u32);
    info!("[HARNESS] sys_write invalid fd returned: {}", ret);
    assert_eq!(ret, err(errno::EBADF));

    // --- sys_write zero length ---
    let msg = b"Hello\0";
    let ret = sys_write(1, msg.as_ptr(), 0);
    info!("[HARNESS] sys_write zero length returned: {}", ret);
    assert_eq!(ret, 0);

    // --- sys_write huge length ---
    let msg = b"Hello\0";
    let ret = sys_write(1, msg.as_ptr(), u32::MAX);
    info!("[HARNESS] sys_write huge length returned: {}", ret);
    assert_eq!(ret, err(errno::EINVAL));

    // --- sys_open happy path ---
    let path = b"foo.txt\0";
    let fd = sys_open(path.as_ptr(), 0);
    info!("[HARNESS] sys_open returned fd: {}", fd);
    assert!(fd >= 3);

    // --- sys_open bogus pointer ---
    let bogus_ptr: u64 = 0xFFFF_FFFF_FFFF_FFFF;
    let fd = unsafe { syscall(SYS_OPEN, bogus_ptr, 0, 0) };
    info!("[HARNESS] sys_open bogus ptr returned fd: {}", fd);
    assert_eq!(fd, err(errno::EFAULT));

    // --- sys_open empty path ---
    let empty = b"\0";
    let fd = sys_open(empty.as_ptr(), 0);
    info!("[HARNESS] sys_open empty path returned fd: {}", fd);
    assert_eq!(fd, err(errno::EINVAL));

    // --- sys_open unsupported flags ---
    let path = b"bar.txt\0";
    let fd = sys_open(path.as_ptr(), 0xFFFF_FFFF);
    info!("[HARNESS] sys_open unsupported flags returned fd: {}", fd);
    assert_eq!(fd, err(errno::EINVAL));

    // --- sys_exit ---
    let code = sys_exit(123);
    info!("[HARNESS] sys_exit returned: {}", code);
    assert_eq!(code, 0);

    // --- sys_exit negative code ---
    let code = sys_exit(u64::MAX);
    info!("[HARNESS] sys_exit negative code returned: {}", code);
    assert_eq!(code, 0);

    // --- unknown syscall ---
    let ret = unsafe { syscall(999, 0, 0, 0) };
    info!("[HARNESS] unknown syscall returned: {}", ret);
    assert_eq!(ret, err(errno::ENOSYS));
}






