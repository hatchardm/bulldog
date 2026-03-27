#![no_std]
#![no_main]
#![allow(invalid_reference_casting)]

use core::panic::PanicInfo;

use uefi::entry;
use uefi::helpers;
use uefi::prelude::*;
use uefi::proto::console::gop::GraphicsOutput;
use uefi::table::boot::{OpenProtocolAttributes, OpenProtocolParams};
use uefi::CStr16;

use boot_proto::{Framebuffer, PixelFormat};

#[entry]
fn efi_main(handle: Handle, mut st: SystemTable<Boot>) -> Status {
    helpers::init(&mut st).expect("UEFI init failed");

    print_marker(&mut st, "Bulldog UEFI bootloader starting...\n");
    print_marker(&mut st, "Reached point A\n");

    {
        // Open GOP
        let gop_scoped = unsafe {
            let bs = st.boot_services();
            bs.open_protocol::<GraphicsOutput>(
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

        // Switch to graphics mode
        {
            let bs = st.boot_services();
            let mode = gop.modes(bs).next().unwrap();
            gop.set_mode(&mode).unwrap();
        }

        // Paint screen blue
        let mode = gop.current_mode_info();
        let mut fb = gop.frame_buffer();

        let (width, height) = mode.resolution();
        let stride = mode.stride();

        let pixel_format = match mode.pixel_format() {
            uefi::proto::console::gop::PixelFormat::Rgb => PixelFormat::Rgb,
            uefi::proto::console::gop::PixelFormat::Bgr => PixelFormat::Bgr,
            _ => PixelFormat::Rgb,
        };

        let fb_info = Framebuffer {
            addr: fb.as_mut_ptr(),
            width,
            height,
            stride,
            pixel_format,
        };

        let fb_ptr = fb_info.addr as *mut u32;
        let blue = match fb_info.pixel_format {
            PixelFormat::Rgb => 0x0000FF,
            PixelFormat::Bgr => 0xFF0000,
            _ => return Status::SUCCESS,
        };

        for y in 0..height {
            let row = unsafe { fb_ptr.add(y * stride) };
            for x in 0..width {
                unsafe { *row.add(x) = blue };
            }
        }
    }

    print_marker(&mut st, "B: After GOP block\n");

    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

fn print_marker(st: &mut SystemTable<Boot>, s: &str) {
    let stdout = st.stdout();
    let mut buf = [0u16; 64];
    let msg = CStr16::from_str_with_buf(s, &mut buf).unwrap();
    let _ = stdout.output_string(msg);
}
