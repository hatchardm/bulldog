//! Bulldog kernel entry point (`main.rs`).

#![no_std]
#![no_main]

use core::panic::PanicInfo;

use boot_proto::BootInfo as ProtoBootInfo;
use kernel::serial::{serial_print, serial_print_u64};

#[unsafe(no_mangle)]
pub extern "sysv64" fn bulldog_entry(boot_info: *mut ProtoBootInfo) -> ! {
    kernel_main(boot_info)
}

#[unsafe(no_mangle)]
pub extern "sysv64" fn kernel_main(boot_info: *mut ProtoBootInfo) -> ! {

 serial_print("KERNEL: entered kernel_main\n");

    let boot: &ProtoBootInfo = unsafe { &*boot_info };

    if boot.framebuffer_present == 0 {
        loop {
            unsafe { core::arch::asm!("hlt"); }
        }
    }

    let fb_ptr = boot.framebuffer.addr as *mut u32;
    let fb_len =
        (boot.framebuffer.stride as usize) * (boot.framebuffer.height as usize);

    let fb = unsafe {
        core::slice::from_raw_parts_mut(fb_ptr, fb_len)
    };

    for pixel in fb.iter_mut() {
        *pixel = 0x00FF00FF; // magenta (BGR)
    }

    loop {
        unsafe { core::arch::asm!("hlt"); }
    }
}


#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    unsafe {
        serial_print("KERNEL PANIC");

        if let Some(location) = info.location() {
            serial_print(" at ");
            serial_print(location.file());
            serial_print(":");
            serial_print_u64(location.line() as u64);
        }

        serial_print("\n");
    }

    loop {
        unsafe { core::arch::asm!("hlt"); }
    }
}









