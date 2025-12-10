//! Syscall stubs for Bulldog kernel.
//! Each stub validates user pointers and returns clean error codes
//! instead of faulting when given bogus addresses.

use log::{info, error};

/// Syscall numbers used by the dispatcher and userland shims.
pub const SYS_WRITE: u64 = 1;
pub const SYS_EXIT:  u64 = 2;
pub const SYS_OPEN:  u64 = 3;

/// Uniform type for syscall functions in the table.
pub type SyscallFn = fn(u64, u64, u64) -> u64;

/// Return placeholder for -EFAULT (until errno is normalized).
#[inline(always)]
fn err_fault() -> u64 { core::u64::MAX }

/// Minimal user-pointer guard: accept only canonical, lower-half addresses.
#[inline(always)]
fn is_user_ptr(ptr: u64) -> bool {
    if ptr == 0 { return false; }
    let canonical = ((ptr as i64) as u64) == ptr;
    canonical && ptr <= 0x0000_7FFF_FFFF_FFFF
}

/// Copy up to `len` bytes from a user pointer into a local buffer.
fn copy_from_user_into(buf_ptr: u64, len: usize, out: &mut [u8]) -> Result<&[u8], ()> {
    if !is_user_ptr(buf_ptr) { return Err(()); }
    let n = core::cmp::min(len, out.len());
    unsafe {
        let src = core::slice::from_raw_parts(buf_ptr as *const u8, n);
        out[..n].copy_from_slice(src);
    }
    Ok(&out[..n])
}

/// Copy a NUL-terminated C string from user memory into a local buffer.
fn copy_cstr_from_user(path_ptr: u64, out: &mut [u8]) -> Result<&str, ()> {
    if !is_user_ptr(path_ptr) { return Err(()); }
    let mut i = 0;
    unsafe {
        while i < out.len() {
            let b = (path_ptr as *const u8).add(i).read();
            out[i] = b;
            if b == 0 { break; }
            i += 1;
        }
    }
    if i == out.len() { return Err(()); }
    core::str::from_utf8(&out[..i]).map_err(|_| ())
}

/// Write syscall: fd, buf_ptr, len
pub fn sys_write(fd: u64, buf_ptr: u64, len: u64) -> u64 {
    let mut scratch = [0u8; 256];
    match copy_from_user_into(buf_ptr, len as usize, &mut scratch) {
        Ok(buf) => {
            let s = core::str::from_utf8(buf).unwrap_or("<invalid utf8>");
            info!("[WRITE] fd={} buf=\"{}\"", fd, s);
            0
        }
        Err(_) => {
            error!("[WRITE] invalid user buffer {:#x}", buf_ptr);
            err_fault()
        }
    }
}

/// Exit syscall: code
pub fn sys_exit(code: u64, _unused1: u64, _unused2: u64) -> u64 {
    info!("[EXIT] process exited with code {}", code);
    0
}

/// Open syscall: path_ptr, flags
pub fn sys_open(path_ptr: u64, flags: u64, _unused: u64) -> u64 {
    let mut scratch = [0u8; 256];
    match copy_cstr_from_user(path_ptr, &mut scratch) {
        Ok(path) => {
            info!("[OPEN] path=\"{}\" flags={}", path, flags);
            42 // dummy fd
        }
        Err(_) => {
            error!("[OPEN] invalid user path ptr {:#x}", path_ptr);
            err_fault()
        }
    }
}



