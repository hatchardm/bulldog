// File: kernel/src/syscall/errno.rs
//! Minimal errno definitions for Bulldog kernel.

pub const EPERM:   u64 = 1;   // Operation not permitted
pub const ENOENT:  u64 = 2;   // No such file or directory
pub const EBADF:   u64 = 9;   // Bad file descriptor
pub const EAGAIN:  u64 = 11;  // Try again
pub const ENOMEM:  u64 = 12;  // Out of memory
pub const EFAULT:  u64 = 14;  // Bad address
pub const EINVAL:  u64 = 22;  // Invalid argument
pub const ENOSYS:  u64 = 38;  // Function not implemented

/// Encode errno as a negative return value (Linux convention).
#[inline(always)]
pub fn err(errno: u64) -> u64 {
    // Cast to i64, negate, then back to u64
    (-(errno as i64)) as u64
}

