// src/syscall/table.rs

//! Static syscall table with function pointer lookup

use super::stubs::{SyscallFn, SYS_WRITE, SYS_OPEN, SYS_EXIT};
use crate::syscall::{write::sys_write, exit::sys_exit, open::sys_open};

pub const SYSCALL_TABLE_SIZE: usize = 512;

const fn init_table() -> [Option<SyscallFn>; SYSCALL_TABLE_SIZE] {
    let mut t: [Option<SyscallFn>; SYSCALL_TABLE_SIZE] = [None; SYSCALL_TABLE_SIZE];
    t[SYS_WRITE as usize] = Some(sys_write);
    t[SYS_EXIT  as usize] = Some(sys_exit);
    t[SYS_OPEN  as usize] = Some(sys_open);
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




