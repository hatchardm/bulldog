// File: kernel/src/syscall/stubs.rs

use log::info;
use core::slice;

// Common syscall function signature: (arg0, arg1, arg2) -> return
pub type SyscallFn = fn(u64, u64, u64) -> u64;

pub const SYS_WRITE: u64 = 1;
pub const SYS_EXIT:  u64 = 2;

// Stub: write to framebuffer/logger
pub fn sys_write(fd: u64, buf_ptr: u64, len: u64) -> u64 {
    info!("sys_write(fd={}, ptr=0x{:x}, len={})", fd, buf_ptr, len);
    unsafe {
        let buf = slice::from_raw_parts(buf_ptr as *const u8, len as usize);
        if let Ok(s) = core::str::from_utf8(buf) {
            info!("Echo from user: {}", s);
        }
    }
    0
}

// Stub: exit process
pub fn sys_exit(code: u64, _unused: u64, _unused2: u64) -> u64 {
    info!("sys_exit(code={})", code);
    // TODO: mark process as terminated
    0
}

