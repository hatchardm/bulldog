use crate::errno::{SysResult, Errno};

#[inline(always)]
pub unsafe fn syscall3(num: u64, a0: u64, a1: u64, a2: u64) -> u64 {
    let ret: u64;
    core::arch::asm!(
        "int 0x80",
        in("rax") num,
        in("rdi") a0,
        in("rsi") a1,
        in("rdx") a2,
        lateout("rax") ret,
        options(nostack, preserves_flags)
    );
    ret
}

#[inline(always)]
pub fn decode_ret(raw: u64) -> SysResult<u64> {
    let signed = raw as i64;
    if signed >= 0 {
        Ok(signed as u64)
    } else {
        Err(Errno::from_raw(-signed))
    }
}