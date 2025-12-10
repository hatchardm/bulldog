//! src/syscall/table.rs
//! Static syscall table with function pointer lookup

use log::warn;
use super::stubs::{SyscallFn, SYS_WRITE, SYS_OPEN, SYS_EXIT, sys_write, sys_exit, sys_open};

/// Size should cover the highest syscall ID you plan to expose.
pub const SYSCALL_TABLE_SIZE: usize = 512;

/// Initialize the syscall table at compile time.
const fn init_table() -> [Option<SyscallFn>; SYSCALL_TABLE_SIZE] {
    let mut t: [Option<SyscallFn>; SYSCALL_TABLE_SIZE] = [None; SYSCALL_TABLE_SIZE];
    t[SYS_WRITE as usize] = Some(sys_write);
    t[SYS_EXIT  as usize] = Some(sys_exit);
    t[SYS_OPEN  as usize] = Some(sys_open);
    t
}

/// Global syscall table, indexed by syscall number.
static SYSCALL_TABLE: [Option<SyscallFn>; SYSCALL_TABLE_SIZE] = init_table();

/// Lookup a syscall function by number.
#[inline]
pub fn lookup(num: u64) -> Option<SyscallFn> {
    let idx = num as usize;
    if idx < SYSCALL_TABLE_SIZE {
        SYSCALL_TABLE[idx]
    } else {
        None
    }
}

/// Central unknown handler so dispatcher stays tidy.
#[inline]
pub fn unknown(num: u64) -> u64 {
    warn!("invalid syscall num={} invoked", num);
    u64::MAX
}


