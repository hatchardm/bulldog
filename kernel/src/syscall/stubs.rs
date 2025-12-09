// File: kernel/src/syscall/stubs.rs

use log::info;
use core::fmt::Write;
use crate::writer::WRITER;

// Common syscall function signature: (arg0, arg1, arg2) -> return
pub type SyscallFn = fn(u64, u64, u64) -> u64;

pub const SYS_WRITE: u64 = 1;
pub const SYS_EXIT:  u64 = 2;
pub const SYS_OPEN:  u64 = 3;

/// Stub: write to framebuffer/logger
/// TODO: actually copy from user buffer once paging/userland is ready
pub fn sys_write(fd: u64, buf_ptr: u64, len: u64) -> u64 {
    // Constrain len to 32 bits if ABI says u32
    let len = (len as usize) & 0xFFFF_FFFF;

    info!("sys_write(fd={}, ptr=0x{:x}, len={})", fd, buf_ptr, len);

    if fd == 1 {
        if let Some(writer) = WRITER.lock().as_mut() {
            let _ = writeln!(writer, "hello bulldog");
        }
        0
    } else {
        u64::MAX
    }
}


/// Stub: exit process
/// TODO: mark process as terminated and trigger scheduler cleanup
pub fn sys_exit(code: u64, _unused: u64, _unused2: u64) -> u64 {
    info!("sys_exit(code={})", code);

    if let Some(writer) = WRITER.lock().as_mut() {
        let _ = writeln!(writer, "process exited with code {}", code);
    }

    0 // success
}

/// Stub: open a file (placeholder).
/// TODO: implement real VFS lookup and flags handling
pub fn sys_open(path_ptr: u64, flags: u64, _unused: u64) -> u64 {
    info!("sys_open(path_ptr=0x{:x}, flags={})", path_ptr, flags);

    if let Some(writer) = WRITER.lock().as_mut() {
        let _ = writeln!(writer, "opened dummy file (fd=42)");
    }

    42 // dummy FD
}


