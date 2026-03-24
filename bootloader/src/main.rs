#![no_std]
#![no_main]

use uefi::prelude::*;

#[entry]
fn efi_main(_handle: Handle, st: SystemTable<Boot>) -> Status {
    // SAFETY: UEFI guarantees the console is valid at boot.
    let stdout = st.stdout();

    // Print a simple message
    let _ = stdout.output_string("Bulldog UEFI bootloader starting...\n");

    Status::SUCCESS
}
