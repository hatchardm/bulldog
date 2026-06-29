#![no_std]
#![no_main]

use boot_proto::BootInfo;

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {
        unsafe { core::arch::asm!("hlt"); }
    }
}

#[unsafe(no_mangle)]
pub extern "sysv64" fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    // Marker: reached kernel_main
    unsafe {
        let mut port = x86_64::instructions::port::Port::new(0x3F8);
        port.write(b'K');
        port.write(b'\n');
    }

    // Call into real kernel entry
    kernel::entry(boot_info);

    // Safety net: if entry ever returns, don’t fall into garbage
    loop {
        unsafe { core::arch::asm!("hlt"); }
    }
}







