#![no_main]
#![no_std]

use uefi::prelude::*;
use log::info;

mod gop;
mod framebuffer;
mod console;
mod color;
mod text;
mod boot;

use gop::init as init_graphics;
use text::load_font;
use console::Console;
use color::Color;
use uefi::proto::console::text::Input;
use uefi::boot as uefi_boot;
use boot::{BootInfo, FramebufferFormat, load_kernel, jump_to_kernel, find_rsdp, acpi_revision};
use uefi::proto::console::gop::PixelFormat;


fn wait_for_keypress() {
    let handle = uefi_boot::get_handle_for_protocol::<Input>()
        .expect("Failed to get handle for Input");

    let mut input = uefi_boot::open_protocol_exclusive::<Input>(handle)
        .expect("Failed to open Input");

    loop {
        if let Ok(Some(_key)) = input.read_key() {
            break;
        }
    }
}

#[entry]
fn main() -> Status {

    uefi::helpers::init().unwrap();
    info!("Bulldog UEFI bootloader starting…");

    // Initialize GOP
    let mut ctx = match init_graphics() {
        Ok(c) => c,
        Err(e) => {
            info!("GOP init failed: {:?}", e);
            return Status::LOAD_ERROR;
        }
    };

    info!(
        "GOP initialized: {}x{} stride {}",
        ctx.fb.width, ctx.fb.height, ctx.fb.stride
    );

    // Snapshot framebuffer info BEFORE borrowing ctx.fb mutably
    let fb_ptr = ctx.fb.data.as_mut_ptr();
    let fb_width = ctx.fb.width;
    let fb_height = ctx.fb.height;
    let fb_stride = ctx.fb.stride;

    let fb_format = match ctx.mode.pixel_format() {
        PixelFormat::Rgb => FramebufferFormat::Rgb,
        PixelFormat::Bgr => FramebufferFormat::Bgr,
        PixelFormat::Bitmask => FramebufferFormat::Bitmask,
        _ => FramebufferFormat::Unknown,
    };

    // RSDP (may be 0 if not found)
    let rsdp_addr = find_rsdp().unwrap_or(0);
    info!("RSDP address: {:#x}", rsdp_addr);

    if let Some(rev) = acpi_revision(rsdp_addr) {
    info!("ACPI revision: {}", rev);
}



    // Load font
    let font = load_font();

    // --- Console scope (borrows &mut ctx.fb) ---
    {
        let mut console = Console::new(&mut ctx.fb, font);
        console.set_color(Color::WHITE);

        console.write_str("Bulldog Bootloader\n");
        console.write_str("Console online.\n");

        console.write_str("Press any key to continue...\n");
        wait_for_keypress();

        console.write_str("Loading kernel...\n");
        let _ = load_kernel(&mut console);

        console.write_str("Preparing kernel handover...\n");
    }
    // --- console dropped here ---

    // Build BootInfo from snapshotted values (no borrow of ctx.fb)
    let boot_info = BootInfo {
        fb_ptr,
        fb_width,
        fb_height,
        fb_stride,
        fb_format,
        rsdp_addr,
    };

    jump_to_kernel(&boot_info);
}


#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}


