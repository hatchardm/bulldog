#![no_std]
#![no_main]

use core::panic::PanicInfo;
use uefi::prelude::*;
use uefi::data_types::CStr16;

use boot_proto::{BootInfo, Framebuffer, PixelFormat, MemoryRegion, MemoryRegionKind};

#[entry]
fn efi_main(_handle: Handle, mut st: SystemTable<Boot>) -> Status {
    let stdout = st.stdout();

    let mut buf = [0u16; 64];
    let msg = CStr16::from_str_with_buf("Bulldog UEFI bootloader starting...\n", &mut buf)
        .unwrap();
    let _ = stdout.output_string(msg);

    // Step 1: just prove we can *type-check* against BootInfo.
    // We’ll fill this in with real GOP + memory map data next.
    let _ = build_boot_info_stub();

    Status::SUCCESS
}

fn build_boot_info_stub() -> BootInfo {
    BootInfo {
        framebuffer: None,
        physical_memory_offset: 0,
        memory_regions: &[],
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}


