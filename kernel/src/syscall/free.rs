use alloc::alloc::{dealloc, Layout};
use crate::syscall::errno::{errno, err};

pub fn sys_free(ptr: usize, size: usize) -> Result<(), u64> {
    if ptr == 0 || size == 0 {
        return Err(errno::EINVAL);
    }

    let layout = Layout::from_size_align(size, 8)
        .map_err(|_| errno::EINVAL)?;

    unsafe { dealloc(ptr as *mut u8, layout) };
    Ok(())
}

pub fn sys_free_trampoline(ptr: u64, size: u64, _a2: u64) -> u64 {
    match sys_free(ptr as usize, size as usize) {
        Ok(()) => 0,
        Err(e) => err(e),
    }
}

