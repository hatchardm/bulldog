// File: src/user_sys.rs
#![allow(dead_code)]

use crate::syscall::{SYS_WRITE, SYS_EXIT, SYS_OPEN};

#[inline(always)]
fn is_err(ret: u64) -> bool {
    ret == core::u64::MAX
}

/// Raw write syscall: fd, ptr, len (len truncated to u32 for ABI consistency).
#[inline(always)]
pub fn write(fd: u64, buf_ptr: u64, len: u64) -> u64 {
    let ret: u64;
    let len32: u32 = len as u32; // ABI uses edx (32-bit)
    unsafe {
        core::arch::asm!(
            "int 0x80",
            in("rax") SYS_WRITE,
            in("rdi") fd,
            in("rsi") buf_ptr,
            in("edx") len32,
            lateout("rax") ret,
            options(nostack, preserves_flags)
        );
    }
    ret
}

/// Convenience: write a &str without manual pointer casting.
#[inline(always)]
pub fn write_str(fd: u64, s: &str) -> u64 {
    write(fd, s.as_ptr() as u64, s.len() as u64)
}

/// Raw exit syscall.
#[inline(always)]
pub fn exit(code: u64) -> u64 {
    let ret: u64;
    unsafe {
        core::arch::asm!(
            "int 0x80",
            in("rax") SYS_EXIT,
            in("rdi") code,
            in("rsi") 0u64,
            in("rdx") 0u64,
            lateout("rax") ret,
            options(nostack, preserves_flags)
        );
    }
    ret
}

/// Raw open syscall: path_ptr must point to a NUL-terminated string.
#[inline(always)]
pub fn open(path_ptr: u64, flags: u64) -> u64 {
    let ret: u64;
    unsafe {
        core::arch::asm!(
            "int 0x80",
            in("rax") SYS_OPEN,
            in("rdi") path_ptr,
            in("rsi") flags,
            in("rdx") 0u64,
            lateout("rax") ret,
            options(nostack, preserves_flags)
        );
    }
    ret
}

/// Convenience: pass a NUL-terminated path literal safely.
/// Returns u64::MAX if `s` is not NUL-terminated (dev-time guard).
#[inline(always)]
pub fn open_cstr(s: &str, flags: u64) -> u64 {
    let bytes = s.as_bytes();
    if bytes.last() != Some(&0) {
        return core::u64::MAX;
    }
    open(bytes.as_ptr() as u64, flags)
}

/// Optional: small helpers for call-site clarity.
#[inline(always)]
pub fn ok(ret: u64) -> bool { !is_err(ret) }
#[inline(always)]
pub fn err() -> u64 { core::u64::MAX }



