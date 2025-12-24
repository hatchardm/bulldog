// src/syscall/table.rs

//! Static syscall table with function pointer lookup

use super::stubs::{SyscallFn, SYS_WRITE, SYS_OPEN, SYS_READ, SYS_EXIT};
use crate::syscall::{write::sys_write, exit::sys_exit, open::sys_open, read::sys_read};

pub const SYSCALL_TABLE_SIZE: usize = 512;

// Trampoline for sys_exit (1-arg) to fit the 3-arg SyscallFn signature.
fn sys_exit_trampoline(code: u64, _arg1: u64, _arg2: u64) -> u64 {
    sys_exit(code)
}

const fn init_table() -> [Option<SyscallFn>; SYSCALL_TABLE_SIZE] {
    let mut t: [Option<SyscallFn>; SYSCALL_TABLE_SIZE] = [None; SYSCALL_TABLE_SIZE];
    t[SYS_WRITE as usize] = Some(sys_write);
    t[SYS_EXIT  as usize] = Some(sys_exit_trampoline);
    t[SYS_OPEN  as usize] = Some(sys_open);
    t[SYS_READ  as usize] = Some(sys_read);
    t
}

static SYSCALL_TABLE: [Option<SyscallFn>; SYSCALL_TABLE_SIZE] = init_table();

#[inline]
pub fn lookup(num: u64) -> Option<SyscallFn> {
    if num < SYSCALL_TABLE_SIZE as u64 {
        SYSCALL_TABLE[num as usize]
    } else {
        None
    }
}





