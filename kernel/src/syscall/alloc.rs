use alloc::alloc::{alloc, Layout};
use crate::syscall::errno::{errno, err};

pub fn sys_alloc(size: usize) -> Result<usize, u64> {
    if size == 0 {
        return Err(errno::EINVAL);
    }

    let layout = Layout::from_size_align(size, 8)
        .map_err(|_| errno::EINVAL)?;

    let ptr = unsafe { alloc(layout) };
    if ptr.is_null() {
        Err(errno::ENOMEM)
    } else {
        Ok(ptr as usize)
    }
}

pub fn sys_alloc_trampoline(size: u64, _a1: u64, _a2: u64) -> u64 {
    match sys_alloc(size as usize) {
        Ok(ptr) => ptr as u64,
        Err(e) => err(e),
    }
}

