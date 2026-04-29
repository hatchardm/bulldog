#![no_main]
#![no_std]

extern crate alloc;

use uefi::prelude::*;
use log::info;

mod gop;
mod framebuffer;
mod console;
mod color;
mod text;
mod boot;

use uefi::boot::{self as uefi_boot, MemoryType};
use uefi::mem::memory_map::MemoryMapOwned;
use x86_64::VirtAddr;

use gop::init as init_graphics;
use text::load_font;
use console::Console;
use color::Color;
use uefi::proto::console::text::Input;
use uefi::proto::console::gop::PixelFormat;

use boot::{
    load_kernel,
    jump_to_kernel,
    find_rsdp,
    acpi_revision,
    fill_memory_regions,
    // init_paging_and_switch_cr3,   // <- not used for this pass
};

use boot_proto::{
    BootInfo as BulldogBootInfo,
    Framebuffer as BulldogFramebuffer,
    PixelFormat as BulldogPixelFormat,
};

const PHYS_MEM_OFFSET: u64 = 0xffff800000000000;

// our own post‑paging stack (unused in this pass)
#[repr(align(16))]
struct AlignedStack([u8; 64 * 1024]);

static mut BOOT_STACK: AlignedStack = AlignedStack([0; 64 * 1024]);

// kernel image location lives in static storage so it survives any future stack switch
static mut KERNEL_PTR: *mut u8 = core::ptr::null_mut();
static mut KERNEL_LEN: usize = 0;

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
                unsafe {
                    KERNEL_PTR = ptr;
                    KERNEL_LEN = len;
                }
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
        physical_memory_offset: PHYS_MEM_OFFSET,
        memory_regions: &[],
    };

    // Fill memory_regions from UEFI memory map
    info!("about to call fill_memory_regions");
    fill_memory_regions(&mut boot_info);
    info!("boot_info.memory_regions.len() = {}", boot_info.memory_regions.len());

    // For this pass: DO NOT touch paging/CR3 at all.
    info!("SKIPPING init_paging_and_switch_cr3 for now");

    let (kernel_ptr, kernel_len) = unsafe { (KERNEL_PTR, KERNEL_LEN) };

    info!(
        "before jump_to_kernel: kernel_ptr = {:#x}, len = {}",
        kernel_ptr as u64,
        kernel_len
    );

    info!("calling jump_to_kernel now");

    jump_to_kernel(kernel_ptr, kernel_len, &mut boot_info);
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}




