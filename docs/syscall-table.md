Syscall Table Example

This document shows how Bulldog wires syscall numbers to stub functions. Contributors can use this as a reference when adding new syscalls.

Syscall Numbers

1 → sys_write

2 → sys_exit

3 → sys_open

Table Implementation

// File: kernel/src/syscall/table.rs

use crate::syscall::stubs::{SYS_WRITE, SYS_EXIT, SYS_OPEN, sys_write, sys_exit, sys_open, SyscallFn};

/// Lookup table mapping syscall numbers to stub functions.
pub fn lookup(num: u64) -> Option<SyscallFn> {
    match num {
        SYS_WRITE => Some(sys_write),
        SYS_EXIT  => Some(sys_exit),
        SYS_OPEN  => Some(sys_open),
        _ => None,
    }
}

/// Fallback for unknown syscalls.
pub fn unknown(num: u64) -> u64 {
    log::warn!("Unknown syscall num={} invoked", num);
    u64::MAX // error code
}

Expected Output

When running with --features syscall_tests, a successful sys_write should produce:

[INFO] Syscall handler initialized at vector 0x80
[INFO] Syscall ready
[INFO] sys_write (fd=1, ptr=0x1, len=1)
hello bulldog
[INFO] syscall num=1 ret=0

Contributor Notes

Add new syscall numbers as constants in stubs.rs.

Implement the stub function with the signature (u64, u64, u64) -> u64.

Extend the lookup match in table.rs to route the new number.

Document expected behavior in docs/syscall.md so others can follow.

This table provides a clear baseline for expanding Bulldog’s syscall harness.