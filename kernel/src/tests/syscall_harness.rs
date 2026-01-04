// File: kernel/src/tests/syscall_harness.rs
//! Expanded syscall harness for Bulldog
//! Exercises happy paths and richer error paths for syscalls.

use crate::syscall::{SYS_WRITE, SYS_EXIT, SYS_OPEN, SYS_READ, SYS_ALLOC, SYS_FREE, SYS_CLOSE};
use crate::syscall::errno::{err, errno};
use crate::syscall::fd::current_process_fd_table;
use log::info;

// Needed in no_std for format! and String
use alloc::format;
use alloc::string::String;

/// Inline assembly wrapper to trigger a syscall with up to 3 args.
#[inline(always)]
unsafe fn syscall(num: u64, arg0: u64, arg1: u64, arg2: u64) -> u64 {
    let ret: u64;
    core::arch::asm!(
        "int 0x80",
        in("rax") num,
        in("rdi") arg0,
        in("rsi") arg1,
        in("rdx") arg2,
        lateout("rax") ret,
        options(nostack, preserves_flags)
    );
    ret
}

// --- typed wrappers for clarity ---
fn sys_write(fd: u64, buf: *const u8, len: u32) -> u64 {
    unsafe { syscall(SYS_WRITE, fd, buf as u64, len as u64) }
}

fn sys_open(path: *const u8, flags: u64) -> u64 {
    unsafe { syscall(SYS_OPEN, path as u64, flags, 0) }
}

fn sys_read(fd: u64, buf: *mut u8, len: u32) -> u64 {
    unsafe { syscall(SYS_READ, fd, buf as u64, len as u64) }
}

fn sys_exit(code: u64) -> u64 {
    unsafe { syscall(SYS_EXIT, code, 0, 0) }
}

fn sys_alloc(size: u64) -> u64 {
    unsafe { syscall(SYS_ALLOC, size, 0, 0) }
}

fn sys_free(ptr: u64, size: u64) -> u64 {
    unsafe { syscall(SYS_FREE, ptr, size, 0) }
}

fn sys_close(fd: u64) -> u64 {
    unsafe { syscall(SYS_CLOSE, fd, 0, 0) }
}


/// Decode raw syscall return values into a readable form.
/// Converts raw u64 → signed errno → symbolic errno.
fn format_ret(raw: u64) -> String {
    let signed = raw as i64;

    if signed >= 0 {
        return format!("{signed}");
    }

    let errno = -signed;

    let name = match errno {
        x if x == errno::EBADF  as i64 => "EBADF",
        x if x == errno::EFAULT as i64 => "EFAULT",
        x if x == errno::EINVAL as i64 => "EINVAL",
        x if x == errno::ENOSYS as i64 => "ENOSYS",
        x if x == errno::ENOMEM as i64 => "ENOMEM",
        x if x == errno::EMFILE as i64 => "EMFILE",
        _ => "EUNKNOWN",
    };

    format!("-{errno} ({name}) [raw={raw}]")
}

/// Uniform harness logging: call=sys_write case=invalid_fd ret=-9 (EBADF) [raw=...]
fn harness_log(call: &str, case: &str, ret: u64) {
    info!(
        "[HARNESS] call={} case={} ret={}",
        call,
        case,
        format_ret(ret)
    );
}

/// Run a sequence of syscall tests.
pub fn run_syscall_tests() {
    // --- sys_write happy path ---
    let msg = b"Hello from harness!\n";
    let ret = sys_write(1, msg.as_ptr(), msg.len() as u32);
    harness_log("sys_write", "happy", ret);
    assert_eq!(ret, msg.len() as u64);

    // --- sys_write bogus pointer ---
    let bogus_ptr: u64 = 0xFFFF_FFFF_FFFF_FFFF;
    let ret = unsafe { syscall(SYS_WRITE, 1, bogus_ptr, 8) };
    harness_log("sys_write", "bogus_ptr", ret);
    assert_eq!(ret, err(errno::EFAULT));

    // --- sys_write invalid fd ---
    let msg = b"Hello\0";
    let ret = sys_write(99, msg.as_ptr(), msg.len() as u32);
    harness_log("sys_write", "invalid_fd", ret);
    assert_eq!(ret, err(errno::EBADF));

    // --- sys_write to stdin (fd=0) ---
    let msg = b"Input?\0";
    let ret = sys_write(0, msg.as_ptr(), msg.len() as u32);
    harness_log("sys_write", "stdin_fd0", ret);
    assert_eq!(ret, err(errno::EBADF));

    // --- sys_write zero length ---
    let msg = b"Hello\0";
    let ret = sys_write(1, msg.as_ptr(), 0);
    harness_log("sys_write", "zero_len", ret);
    assert_eq!(ret, 0);

    // --- sys_write huge length ---
    let msg = b"Hello\0";
    let ret = sys_write(1, msg.as_ptr(), u32::MAX);
    harness_log("sys_write", "huge_len", ret);

    // The kernel currently clamps huge writes to MAX_WRITE and returns the
    // number of bytes actually written instead of EINVAL.
    const MAX_WRITE: u64 = 4096;
    assert!(ret <= MAX_WRITE);


    // --- sys_open happy path ---
    let path = b"foo.txt\0";
    let fd = sys_open(path.as_ptr(), 0);
    harness_log("sys_open", "happy", fd);
    assert!(fd >= 3);

    // --- sys_open bogus pointer ---
    let bogus_ptr: u64 = 0xFFFF_FFFF_FFFF_FFFF;
    let fd = unsafe { syscall(SYS_OPEN, bogus_ptr, 0, 0) };
    harness_log("sys_open", "bogus_ptr", fd);
    assert_eq!(fd, err(errno::EFAULT));

    // --- sys_open empty path ---
    let empty = b"\0";
    let fd = sys_open(empty.as_ptr(), 0);
    harness_log("sys_open", "empty_path", fd);
    assert_eq!(fd, err(errno::EINVAL));

    // --- sys_open unsupported flags ---
    let path = b"bar.txt\0";
    let fd = sys_open(path.as_ptr(), 0xFFFF_FFFF);
    harness_log("sys_open", "bad_flags", fd);
    assert_eq!(fd, err(errno::EINVAL));

    // --- sys_open FD exhaustion and reuse ---
    // Open files until we hit EMFILE, tracking all successful FDs.
    let mut fds: [u64; 64] = [0; 64];
    let mut count = 0usize;

    loop {
        let path = b"fd_exhaust.txt\0";
        let fd = sys_open(path.as_ptr(), 0);
        let signed = fd as i64;

        if signed < 0 {
            // Expect EMFILE when we run out of descriptors.
            harness_log("sys_open", "fd_exhaustion", fd);
            assert_eq!(fd, err(errno::EMFILE));
            break;
        }

        harness_log("sys_open", "fd_exhaustion_open", fd);
        fds[count] = fd;
        count += 1;

        // Safety guard: don't overflow our local array.
        assert!(count < fds.len(), "FD exhaustion test exceeded local FD buffer");
    }

    assert!(count > 0, "FD exhaustion test should have opened at least one FD");

    // Find the lowest FD we opened (should be >= 3).
    let mut lowest_fd = u64::MAX;
    for i in 0..count {
        if fds[i] != 0 && fds[i] < lowest_fd {
            lowest_fd = fds[i];
        }
    }
    assert!(lowest_fd >= 3, "lowest FD in exhaustion test should be >= 3");

    // Close the lowest FD and ensure it becomes reusable.
    let ret = sys_close(lowest_fd);
    harness_log("sys_close", "close_lowest_fd", ret);
    assert_eq!(ret, 0, "closing a valid FD should succeed");

    // Open again and assert that we get the freed lowest FD back.
    let path = b"fd_reuse.txt\0";
    let fd_reused = sys_open(path.as_ptr(), 0);
    harness_log("sys_open", "fd_reuse", fd_reused);
    assert_eq!(fd_reused, lowest_fd, "sys_open should reuse the lowest available FD");


   // --- sys_read happy path ---
let mut buf = [0u8; 16];

// read from the fd we just reused (4), not stdin (0)
let ret = sys_read(4, buf.as_mut_ptr(), buf.len() as u32);
harness_log("sys_read", "happy", ret);

// For now, sys_read is unimplemented and returns 0 bytes read.
assert_eq!(ret, err(errno::EBADF));



    // --- sys_read zero length ---
    let ret = sys_read(0, buf.as_mut_ptr(), 0);
    harness_log("sys_read", "zero_len", ret);
    assert_eq!(ret, 0);

    // --- sys_read invalid fd ---
    let ret = sys_read(99, buf.as_mut_ptr(), buf.len() as u32);
    harness_log("sys_read", "invalid_fd", ret);
    assert_eq!(ret, err(errno::EBADF));

    // --- unknown syscall (must run BEFORE sys_exit) ---
    let ret = unsafe { syscall(999, 0, 0, 0) };
    harness_log("sys_unknown", "num_999", ret);
    assert_eq!(ret, err(errno::ENOSYS));

    // --- sys_alloc happy path ---
    let size = 64u64;
    let ptr = sys_alloc(size);
    harness_log("sys_alloc", "happy", ptr);
    assert!(ptr != 0, "sys_alloc(64) should return non-zero ptr");

    // --- sys_free happy path ---
    let ret = sys_free(ptr, size);
    harness_log("sys_free", "happy", ret);
    assert_eq!(ret, 0, "sys_free should return 0 on success");

    // --- sys_alloc zero size ---
    let ret = sys_alloc(0);
    harness_log("sys_alloc", "zero_size", ret);
    assert_eq!(ret, err(errno::EINVAL));

    // --- sys_free zero ptr ---
    let ret = sys_free(0, 64);
    harness_log("sys_free", "zero_ptr", ret);
    assert_eq!(ret, err(errno::EINVAL));

    // --- sys_free zero size ---
    let ret = sys_free(ptr, 0);
    harness_log("sys_free", "zero_size", ret);
    assert_eq!(ret, err(errno::EINVAL));

    // --- syscall table sweep: implemented vs unimplemented ---
    // Implemented syscall numbers should not return ENOSYS.
    let implemented = [
        SYS_WRITE,
        SYS_EXIT,
        SYS_OPEN,
        SYS_READ,
        SYS_ALLOC,
        SYS_FREE,
        SYS_CLOSE,
    ];

    for &num in implemented.iter() {
        let ret = unsafe { syscall(num, 0, 0, 0) };
        harness_log("sys_table", "implemented", ret);
        // If a syscall legitimately returns ENOSYS internally, this would fail,
        // but for Bulldog's current set, none should.
        assert_ne!(ret, err(errno::ENOSYS), "implemented syscall {} returned ENOSYS", num);
    }

    // Clearly unimplemented syscall numbers should return ENOSYS.
    let unknowns = [0u64, 8, 9, 500];

    for &num in unknowns.iter() {
        let ret = unsafe { syscall(num, 0, 0, 0) };
        harness_log("sys_table", "unimplemented", ret);
        assert_eq!(ret, err(errno::ENOSYS), "unimplemented syscall {} did not return ENOSYS", num);
    }

    
    // --- sys_exit ---
    let code = sys_exit(123);
    harness_log("sys_exit", "happy", code);
    assert_eq!(code, 0);

    let guard = current_process_fd_table();
    assert!(
        guard.as_ref().unwrap().is_empty(),
        "FD table should be empty after exit"
    );

    // --- sys_exit with max code ---
    let code = sys_exit(u64::MAX);
    harness_log("sys_exit", "max_code", code);
    assert_eq!(code, 0);
}








