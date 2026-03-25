#![no_std]
#![no_main]

use core::panic::PanicInfo;
use uefi::prelude::*;
use uefi::data_types::CStr16;
use uefi::proto::console::gop::GraphicsOutput;
use uefi::helpers;

use boot_proto::{Framebuffer, PixelFormat};

#[entry]
fn efi_main() -> Status {
    // Initialise UEFI helper system (sets up global system table, logging, etc.)
    helpers::init().expect("UEFI init failed");

    // Get the system table from helpers
    let mut st = helpers::system_table();

    // First message
    {
        let stdout = st.stdout();
        let mut buf = [0u16; 64];
        let msg =
            CStr16::from_str_with_buf("Bulldog UEFI bootloader starting...\n", &mut buf).unwrap();
        let _ = stdout.output_string(msg);
    }

    // Locate GOP safely
    let gop = unsafe {
        st.boot_services()
            .locate_protocol::<GraphicsOutput>()
            .expect("Failed to locate GOP")
            .get()
            .expect("GOP pointer was null")
    };

    let mode = gop.current_mode_info();
    let mut fb = gop.frame_buffer();

    let (width, height) = mode.resolution();
    let stride = mode.stride();

    let pixel_format = match mode.pixel_format() {
        uefi::proto::console::gop::PixelFormat::Rgb => PixelFormat::Rgb,
        uefi::proto::console::gop::PixelFormat::Bgr => PixelFormat::Bgr,
        _ => PixelFormat::Rgb,
    };

    let _fb_info = Framebuffer {
        addr: fb.as_mut_ptr(),
        width,
        height,
        stride,
        pixel_format,
    };

    // Second message
    {
        let stdout = st.stdout();
        let mut buf2 = [0u16; 128];
        let msg2 =
            CStr16::from_str_with_buf("GOP OK: framebuffer extracted.\n", &mut buf2).unwrap();
        let _ = stdout.output_string(msg2);
    }

    Status::SUCCESS
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}












