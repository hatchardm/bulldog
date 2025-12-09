// File: src/user_sys.rs
#![allow(dead_code)]
#[inline(always)]
pub fn write(fd: u64, buf_ptr: u64, len: u64) -> u64 {
    let ret: u64;
    unsafe {
        core::arch::asm!(
            "int 0x80",
            in("rax") 1u64,
            in("rdi") fd,
            in("rsi") buf_ptr,
            in("rdx") len,
            lateout("rax") ret,
            options(nostack, preserves_flags)
        );
    }
    ret
}

#[inline(always)]
pub fn exit(code: u64) -> u64 {
    let ret: u64;
    unsafe {
        core::arch::asm!(
            "int 0x80",
            in("rax") 2u64,
            in("rdi") code,
            in("rsi") 0u64,
            in("rdx") 0u64,
            lateout("rax") ret,
            options(nostack, preserves_flags)
        );
    }
    ret
}

#[inline(always)]
pub fn open(path_ptr: u64, flags: u64) -> u64 {
    let ret: u64;
    unsafe {
        core::arch::asm!(
            "int 0x80",
            in("rax") 3u64,
            in("rdi") path_ptr,
            in("rsi") flags,
            in("rdx") 0u64,
            lateout("rax") ret,
            options(nostack, preserves_flags)
        );
    }
    ret
}


