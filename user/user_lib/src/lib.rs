#![no_std]

use core::arch::asm;

pub const SYS_WRITE: u64 = 1;
pub const SYS_EXIT: u64 = 2;

pub fn sys_write(fd: u64, buf: *const u8, len: u64) -> i64 {
    let mut ret: u64;
    unsafe {
        asm!(
            "int 0x80",
            inlateout("rax") SYS_WRITE => ret,
            in("rdi") fd,
            in("rsi") buf,
            in("rdx") len,
            options(nostack, preserves_flags)
        );
    }
    ret as i64
}

pub fn sys_exit(code: u64) -> ! {
    unsafe {
        asm!(
            "int 0x80",
            in("rax") SYS_EXIT,
            in("rdi") code,
            options(noreturn, nostack)
        );
    }
}