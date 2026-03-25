#![no_std]
#![no_main]

use core::panic::PanicInfo;
use uefi::prelude::*;
use uefi::data_types::CStr16;
use uefi::proto::console::gop::GraphicsOutput;
use uefi::table::boot::{OpenProtocolAttributes, OpenProtocolParams};

use boot_proto::{Framebuffer, PixelFormat};

#[entry]
fn efi_main(handle: Handle, mut st: SystemTable<Boot>) -> Status {
    // First message: keep stdout borrow in a small scope
    {
        let stdout = st.stdout();
        let mut buf = [0u16; 64];
        let msg =
            CStr16::from_str_with_buf("Bulldog UEFI bootloader starting...\n", &mut buf).unwrap();
        let _ = stdout.output_string(msg);
    }

    // Open GOP and keep the ScopedProtocol alive in this scope
    let gop_scoped = unsafe {
        st.boot_services()
            .open_protocol::<GraphicsOutput>(
                OpenProtocolParams {
                    handle,
                    agent: handle,
                    controller: None,
                },
                OpenProtocolAttributes::Exclusive,
            )
            .expect("Failed to open GOP")
    };

    let gop_ref = gop_scoped.get().expect("GOP pointer was null");

    // We know we're the only user, so cast &T -> &mut T here.
    let gop: &mut GraphicsOutput =
        unsafe { &mut *(gop_ref as *const _ as *mut GraphicsOutput) };

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

    // Second message: take a fresh stdout borrow
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






