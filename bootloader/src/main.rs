#![no_std]
#![no_main]

use log::info;
use uefi::prelude::*;
use uefi::println;
use uefi::proto::console::gop::GraphicsOutput;
use uefi::boot;                    // keep this
use uefi::mem::memory_map::MemoryType;

mod color;
mod framebuffer;
mod console;
mod text;
mod boot_mod;                      // ← renamed file/module
mod paging;

use framebuffer::Framebuffer;
use console::Console;
use color::Color;
use text::load_font;

use boot_mod::{load_kernel, fill_boot_info, jump_to_kernel};
use boot_proto::BootInfo;

#[entry]
fn efi_main() -> Status {
    // --------------------------------------------------
    // 1. UEFI helpers
    // --------------------------------------------------
    uefi::helpers::init().unwrap();

    println!("Bulldog bootloader starting...\n");
    info!("Bulldog bootloader starting...");

    // --------------------------------------------------
    // 2. Graphics Output Protocol
    // --------------------------------------------------
    let gop_handle = boot::get_handle_for_protocol::<GraphicsOutput>()
        .expect("No GraphicsOutput protocol found");

    let mut gop = boot::open_protocol_exclusive::<GraphicsOutput>(gop_handle)
        .expect("Failed to open GOP");

    let mode = gop.current_mode_info();
    let mut fb_view = gop.frame_buffer();

    let (width, height) = mode.resolution();
    let stride = mode.stride();
    let pixel_format = mode.pixel_format();

    println!(
        "GOP mode: {}x{}, stride = {}, pixel_format = {:?}",
        width, height, stride, pixel_format
    );

    // --------------------------------------------------
    // 3. Framebuffer + Console
    // --------------------------------------------------
    let mut fb = Framebuffer::new(&mut fb_view, width, height, stride);
    let font = load_font();
    let mut console = Console::new(&mut fb, font);

    console.set_color(Color::WHITE);
    console.write_str("Bulldog bootloader\n");
    console.write_str("Framebuffer console is alive!\n\n");

    // --------------------------------------------------
    // 4. Load kernel.elf + fat.img
    // --------------------------------------------------
    let (kernel_ptr, kernel_size, fat_phys, fat_size) =
        match load_kernel(&mut console) {
            Ok(v) => v,
            Err(status) => {
                console.set_color(Color::RED);
                console.write_str("Failed to load kernel / fat.img\n");
                return status;
            }
        };

    // --------------------------------------------------
    // 5. Fill BootInfo
    // --------------------------------------------------
       console.set_color(Color::GREEN);
    console.write_str("BootInfo filled.\n");
    console.write_str("Exiting boot services...\n");

    // Drop console so we can use fb later if needed
    drop(console);

    let mut boot_info = unsafe { core::mem::zeroed::<BootInfo>() };
    fill_boot_info(&mut boot_info, &fb, pixel_format, fat_phys, fat_size);

    // --------------------------------------------------
    // 6. Exit boot services
    // --------------------------------------------------
    let mmap = unsafe { boot::exit_boot_services(Some(MemoryType::LOADER_DATA)) };



    // --------------------------------------------------
    // 7. Set up paging
    // --------------------------------------------------
    paging::setup_minimal_paging(&mmap);



    // --------------------------------------------------
    // 8. Jump to kernel
    // --------------------------------------------------
    jump_to_kernel(kernel_ptr, kernel_size, &mut boot_info);
}




