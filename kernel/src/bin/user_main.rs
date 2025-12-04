// src/bin/user_main.rs
#![no_std]
#![no_main]

use log::info;
use core::panic::PanicInfo;
use kernel::user_sys;

fn main() {
    let msg = "hello bulldog";
    info!("Triggering sys_write...");
    let ret = user_sys::write(1, msg.as_ptr() as u64, msg.len() as u64);
    info!("sys_write returned {}", ret);

    info!("Triggering sys_exit...");
    let _ = user_sys::exit(0);
    info!("Returned from sys_exit.");
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
