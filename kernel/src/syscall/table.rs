// src/syscall/table.rs

//! Static syscall table with function pointer lookup

use super::stubs::{
    SyscallFn,
    SYS_WRITE,
    SYS_OPEN,
    SYS_READ,
    SYS_EXIT,
    SYS_ALLOC,
    SYS_FREE,
    SYS_CLOSE,
};

use crate::syscall::{
    write::sys_write,
    exit::sys_exit,
    open::sys_open,
    read::sys_read,
    alloc::sys_alloc_trampoline,
    free::sys_free_trampoline,
    close::sys_close,
};

pub const SYSCALL_TABLE_SIZE: usize = 512;

// Trampoline for sys_exit (1-arg) to fit the 3-arg SyscallFn signature.
fn sys_exit_trampoline(code: u64, _arg1: u64, _arg2: u64) -> u64 {
    sys_exit(code)
}

// Trampoline for sys_close (1-arg) to fit the 3-arg SyscallFn signature.
fn sys_close_trampoline(fd: u64, _arg1: u64, _arg2: u64) -> u64 {
    sys_close(fd)
}

const fn init_table() -> [Option<SyscallFn>; SYSCALL_TABLE_SIZE] {
    let mut t: [Option<SyscallFn>; SYSCALL_TABLE_SIZE] = [None; SYSCALL_TABLE_SIZE];

    t[SYS_WRITE as usize] = Some(sys_write);
    t[SYS_EXIT  as usize] = Some(sys_exit_trampoline);
    t[SYS_OPEN  as usize] = Some(sys_open);
    t[SYS_READ  as usize] = Some(sys_read);

    // Allocator syscalls (ABI trampolines)
    t[SYS_ALLOC as usize] = Some(sys_alloc_trampoline);
    t[SYS_FREE  as usize] = Some(sys_free_trampoline);
    t[SYS_CLOSE as usize] = Some(sys_close_trampoline);
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






