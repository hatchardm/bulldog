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
use uefi::proto::console::gop::PixelFormat;
use boot::{load_kernel, jump_to_kernel, find_rsdp, acpi_revision, fill_memory_regions};

use boot_proto::{
    BootInfo as BulldogBootInfo,
    Framebuffer as BulldogFramebuffer,
    PixelFormat as BulldogPixelFormat,
};

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
    info!("*** BULLDOG BOOTLOADER START v999 ***");

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

    // RSDP (may be 0 if not found)
    let rsdp_addr = find_rsdp().unwrap_or(0);
    info!("RSDP address: {:#x}", rsdp_addr);

    if let Some(rev) = acpi_revision(rsdp_addr) {
        info!("ACPI revision: {}", rev);
    }

    // Load font
    let font = load_font();

    // --- Console scope (borrows &mut ctx.fb) ---
    let mut kernel_ptr: *mut u8 = core::ptr::null_mut();
    let mut kernel_len: usize = 0;

    {
        let mut console = Console::new(&mut ctx.fb, font);
        console.set_color(Color::WHITE);

        console.write_str("Bulldog Bootloader\n");
        console.write_str("Console online.\n");

        console.write_str("Press any key to continue...\n");
        wait_for_keypress();

        console.write_str("Loading kernel...\n");
        match load_kernel(&mut console) {
            Ok((ptr, len)) => {
                kernel_ptr = ptr;
                kernel_len = len;
            }
            Err(_) => {
                console.write_str("Kernel load failed.\n");
                return Status::LOAD_ERROR;
            }
        }

        console.write_str("Preparing kernel handover...\n");
    }
    // --- console dropped here ---

    // Build BulldogBootInfo from snapshotted values
    let framebuffer = BulldogFramebuffer {
        addr: fb_ptr,
        width: fb_width,
        height: fb_height,
        stride: fb_stride,
        pixel_format: match ctx.mode.pixel_format() {
            PixelFormat::Rgb => BulldogPixelFormat::Rgb,
            PixelFormat::Bgr => BulldogPixelFormat::Bgr,
            PixelFormat::Bitmask => BulldogPixelFormat::Bitmask,
            _ => BulldogPixelFormat::BltOnly,
        },
    };

    let mut boot_info = BulldogBootInfo {
        framebuffer: Some(framebuffer),
        physical_memory_offset: 0,
        memory_regions: &[],
    };

    // Fill memory_regions from UEFI memory map
    info!("about to call fill_memory_regions");
fill_memory_regions(&mut boot_info);
info!("boot_info.memory_regions.len() = {}", boot_info.memory_regions.len());

info!(
    "calling jump_to_kernel: ptr = {:#x}, len = {}",
    kernel_ptr as u64,
    kernel_len
);




jump_to_kernel(kernel_ptr, kernel_len, &mut boot_info);

}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}



