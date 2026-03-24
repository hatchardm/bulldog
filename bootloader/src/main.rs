#![no_std]
#![no_main]

use uefi::prelude::*;
use uefi::data_types::CStr16;
use core::panic::PanicInfo;

#[entry]
fn efi_main(_handle: Handle, mut st: SystemTable<Boot>) -> Status {
    let stdout = st.stdout();

    let mut buf = [0u16; 64];
    let msg = CStr16::from_str_with_buf("Bulldog UEFI bootloader starting...\n", &mut buf)
        .unwrap();

    let _ = stdout.output_string(msg);

    Status::SUCCESS
}


#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

