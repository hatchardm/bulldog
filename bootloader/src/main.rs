#![no_std]
#![no_main]
#![allow(invalid_reference_casting)]

use core::panic::PanicInfo;
use uefi::prelude::*;
use uefi::data_types::CStr16;
use uefi::proto::console::gop::GraphicsOutput;
use uefi::table::boot::{OpenProtocolAttributes, OpenProtocolParams};
use uefi::helpers;

use boot_proto::{Framebuffer, PixelFormat};

#[entry]
fn efi_main(handle: Handle, mut st: SystemTable<Boot>) -> Status {
    helpers::init(&mut st).expect("UEFI init failed");

    // First message
    {
        let stdout = st.stdout();
        let mut buf = [0u16; 64];
        let msg =
            CStr16::from_str_with_buf("Bulldog UEFI bootloader starting...\n", &mut buf).unwrap();
        let _ = stdout.output_string(msg);
    }

    // Open GOP
    let gop_scoped = unsafe {
        st.boot_services()
            .open_protocol::<GraphicsOutput>(
                OpenProtocolParams {
                    handle,
                    agent: handle,
                    controller: None,
                },
                OpenProtocolAttributes::GetProtocol,
            )
            .expect("Failed to open GOP")
    };

    let gop_ref = gop_scoped.get().expect("GOP pointer was null");
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

    // ⭐ NOW paint the screen blue — after _fb_info exists
    {
        let fb_ptr = _fb_info.addr as *mut u32;
        let width = _fb_info.width;
        let height = _fb_info.height;
        let stride = _fb_info.stride;

        let blue = match _fb_info.pixel_format {
    PixelFormat::Rgb => 0x0000FF,
    PixelFormat::Bgr => 0xFF0000,
    _ => return Status::SUCCESS, // unsupported formats: just exit cleanly
};


        for y in 0..height {
            let row = unsafe { fb_ptr.add(y * stride) };
            for x in 0..width {
                unsafe { *row.add(x) = blue };
            }
        }
    }

    Status::SUCCESS
}


#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}













