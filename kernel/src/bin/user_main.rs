
//! src/bin/user_main.rs
//! Temporary userland harness to exercise syscalls via int 0x80.
//! Replace with proper Ring 3 task once TSS/GDT is wired.

#![no_std]
#![no_main]

use log::info;
use core::panic::PanicInfo;
use kernel::user_sys;

fn main() {
    // Write a message using the safe wrapper
    info!("Triggering sys_write...");
    let ret = user_sys::write_str(1, "hello bulldog");
    if ret == core::u64::MAX {
        info!("sys_write failed (EFAULT/EINVAL placeholder)");
    } else {
        info!("sys_write returned {}", ret);
    }

    // Open with a proper NUL-terminated path using the wrapper
    info!("Triggering sys_open...");
    let fd = user_sys::open_cstr("foo.txt\0", 0);
    if fd == core::u64::MAX {
        info!("sys_open failed (EFAULT placeholder)");
    } else {
        info!("sys_open returned {}", fd);
    }

    // Exit last
    info!("Triggering sys_exit...");
    let _ = user_sys::exit(0);
    info!("Returned from sys_exit (stubbed behavior).");
}


#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

