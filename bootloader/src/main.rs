#![no_std]
#![no_main]

use uefi::prelude::*;
use uefi::CString16;
use core::panic::PanicInfo;

#[entry]
fn efi_main(_handle: Handle, mut st: SystemTable<Boot>) -> Status {
    let stdout = st.stdout();

    let msg = CString16::try_from("Bulldog UEFI bootloader starting...\n").unwrap();
    let _ = stdout.output_string(&msg);


    Status::SUCCESS
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

