// user_sys.rs
//! Minimal user-side syscall wrappers using int 0x80.
//! ABI (temporary): rax = num, rdi/rsi/rdx = arg0/arg1/arg2, return in rax.

#[inline(always)]
pub fn write(fd: u64, buf_ptr: u64, len: u64) -> u64 {
    let ret: u64;
    unsafe {
        core::arch::asm!(
            "int 0x80",
            in("rax") 1u64,         // SYS_WRITE
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
            in("rax") 2u64,         // SYS_EXIT
            in("rdi") code,
            lateout("rax") ret,
            options(nostack, preserves_flags)
        );
    }
    ret
}
