//! Syscall stubs and helpers for Bulldog kernel.
//! Provides syscall numbers, type alias, and user-pointer validation utilities.
//! Actual syscall implementations live in their own modules (write.rs, exit.rs, open.rs).

/// Syscall numbers used by the dispatcher and userland shims.
pub const SYS_WRITE: u64 = 1;
pub const SYS_EXIT:  u64 = 2;
pub const SYS_OPEN:  u64 = 3;
pub const SYS_READ:  u64 = 4;
pub const SYS_ALLOC: u64 = 5;
pub const SYS_FREE:  u64 = 6;
pub const SYS_CLOSE: u64 = 7;


/// Uniform type for syscall functions in the table.
pub type SyscallFn = fn(u64, u64, u64) -> u64;

/// Minimal user-pointer guard: accept only canonical, lower-half addresses.
#[inline(always)]
pub fn is_user_ptr(ptr: u64) -> bool {
    if ptr == 0 { return false; }
    let canonical = ((ptr as i64) as u64) == ptr;
    canonical && ptr <= 0x0000_7FFF_FFFF_FFFF
}

/// Copy up to `len` bytes from a user pointer into a local buffer.
pub fn copy_from_user_into(buf_ptr: u64, len: usize, out: &mut [u8]) -> Result<&[u8], ()> {
    if !is_user_ptr(buf_ptr) { return Err(()); }
    let n = core::cmp::min(len, out.len());
    unsafe {
        let src = core::slice::from_raw_parts(buf_ptr as *const u8, n);
        out[..n].copy_from_slice(src);
    }
    Ok(&out[..n])
}

/// Copy a NUL-terminated C string from user memory into a local buffer.
pub fn copy_cstr_from_user(path_ptr: u64, out: &mut [u8]) -> Result<&str, ()> {
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




