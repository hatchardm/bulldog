use crate::errno::{SysResult, Errno};
use crate::raw::{syscall3, decode_ret};

pub const SYS_WRITE: u64 = 1;
pub const SYS_EXIT:  u64 = 2;
pub const SYS_OPEN:  u64 = 3;
pub const SYS_READ:  u64 = 4;
pub const SYS_ALLOC: u64 = 5;
pub const SYS_FREE:  u64 = 6;
pub const SYS_CLOSE: u64 = 7;

// --- write ---
pub fn write(fd: u64, buf: &[u8]) -> SysResult<u64> {
    let raw = unsafe { syscall3(SYS_WRITE, fd, buf.as_ptr() as u64, buf.len() as u64) };
    decode_ret(raw)
}

// --- read ---
pub fn read(fd: u64, buf: &mut [u8]) -> SysResult<u64> {
    let raw = unsafe { syscall3(SYS_READ, fd, buf.as_mut_ptr() as u64, buf.len() as u64) };
    decode_ret(raw)
}

// --- open ---
pub fn open(path: &str, flags: u64) -> SysResult<u64> {
    use alloc::vec::Vec;

    let mut bytes = Vec::with_capacity(path.len() + 1);
    bytes.extend_from_slice(path.as_bytes());
    bytes.push(0);

    let raw = unsafe { syscall3(SYS_OPEN, bytes.as_ptr() as u64, flags, 0) };
    decode_ret(raw)
}

// --- close ---
pub fn close(fd: u64) -> SysResult<()> {
    let raw = unsafe { syscall3(SYS_CLOSE, fd, 0, 0) };
    decode_ret(raw).map(|_| ())
}

// --- exit ---
pub fn exit(code: u64) -> ! {
    let _ = unsafe { syscall3(SYS_EXIT, code, 0, 0) };
    loop {
        core::hint::spin_loop();
    }
}