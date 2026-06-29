#![no_main]
#![no_std]

extern crate alloc;

use uefi::prelude::*;
use uefi::boot as uefi_boot;
use uefi::proto::console::text::Input;
use uefi::proto::console::gop::PixelFormat;

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

use boot::{
    load_kernel,
    jump_to_kernel,
    find_rsdp,
    acpi_revision,
    fill_memory_regions_from_map,
    init_paging_and_switch_cr3,
};

use boot_proto::{
    BootInfo as BulldogBootInfo,
    Framebuffer as BulldogFramebuffer,
    PixelFormat as BulldogPixelFormat,
};

const PHYS_MEM_OFFSET: u64 = 0xffff800000000000;

#[repr(align(16))]
struct AlignedStack([u8; 64 * 1024]);

static mut BOOT_STACK: AlignedStack = AlignedStack([0; 64 * 1024]);

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

    // -------------------------------
    // Initialize GOP
    // -------------------------------
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

    // Snapshot framebuffer BEFORE mutable borrow
    let fb_addr_phys = ctx.fb.data.as_mut_ptr() as u64;
    let fb_width = ctx.fb.width;
    let fb_height = ctx.fb.height;
    let fb_stride = ctx.fb.stride;

    // -------------------------------
    // ACPI
    // -------------------------------
    let rsdp_addr = find_rsdp().unwrap_or(0);
    info!("RSDP address: {:#x}", rsdp_addr);

    if let Some(rev) = acpi_revision(rsdp_addr) {
        info!("ACPI revision: {}", rev);
    }

    // -------------------------------
    // Console + Kernel Load
    // -------------------------------
    info!("about to load_font");
    let font = load_font();
    info!("load_font returned");

    info!("about to create Console");
    {
        let mut console = Console::new(&mut ctx.fb, font);
        info!("Console::new returned");

        console.set_color(Color::WHITE);

        console.write_str("Bulldog Bootloader\n");
        console.write_str("Console online.\n");

        console.write_str("Press any key to continue...\n");
        wait_for_keypress();

        console.write_str("Loading kernel...\n");
        match load_kernel(&mut console) {
            Ok((ptr, len)) => unsafe {
                KERNEL_PTR = ptr;
                KERNEL_LEN = len;
            },
            Err(_) => {
                console.write_str("Kernel load failed.\n");
                return Status::LOAD_ERROR;
            }
        }

        console.write_str("Preparing kernel handover...\n");
    }

    // -------------------------------
    // Build BootInfo
    // -------------------------------
    let framebuffer = BulldogFramebuffer {
        addr: fb_addr_phys,
        width: fb_width as u32,
        height: fb_height as u32,
        stride: fb_stride as u32,
        pixel_format: match ctx.mode.pixel_format() {
            PixelFormat::Rgb => BulldogPixelFormat::Rgb,
            PixelFormat::Bgr => BulldogPixelFormat::Bgr,
            PixelFormat::Bitmask => BulldogPixelFormat::Bitmask,
            _ => BulldogPixelFormat::BltOnly,
        },
    };

    let mut boot_info = BulldogBootInfo {
        framebuffer,
        framebuffer_present: 1,
        _pad0: [0; 7],
        physical_memory_offset: PHYS_MEM_OFFSET,
        memory_regions: core::ptr::null(),
        memory_region_count: 0,
    };

    info!("Framebuffer phys addr: {:#x}", fb_addr_phys);

    // -------------------------------
    // Exit Boot Services
    // -------------------------------
    info!("about to call ExitBootServices");

    let memory_map_owned = unsafe { uefi_boot::exit_boot_services(None) };


    info!("ExitBootServices returned");

    // -------------------------------
    // Paging + BootInfo memory map
    // -------------------------------
    // 1) Set up HHDM mappings in the existing PML4 (no CR3 change)

 
    unsafe {
    let mut port = x86_64::instructions::port::Port::new(0x3F8);
    port.write(b'P');   // before paging init
    port.write(b'\n');
}
/* 
unsafe {
    boot::init_paging_and_switch_cr3(
        &memory_map_owned,
        x86_64::VirtAddr::new(PHYS_MEM_OFFSET),
        unsafe { KERNEL_PTR },
        unsafe { KERNEL_LEN },
    );
}
*/
unsafe {
    let mut port = x86_64::instructions::port::Port::new(0x3F8);
    port.write(b'p');   // after paging init
    port.write(b'\n');
}


    // 2) Fill BootInfo.memory_regions from the UEFI memory map
    fill_memory_regions_from_map(&mut boot_info, &memory_map_owned);

    // -------------------------------
    // TEMP: prove we’re alive after paging setup
    // -------------------------------
    let fb_ptr = fb_addr_phys as *mut u32;
    let fb_len = (fb_stride as usize) * (fb_height as usize);

    unsafe {
        let fb = core::slice::from_raw_parts_mut(fb_ptr, fb_len);
        for pixel in fb.iter_mut() {
            *pixel = 0x00000000; // black
        }
        for pixel in fb.iter_mut() {
            *pixel = 0x0000FF00; // green (BGR)
        }
    }

    // -------------------------------
    // Jump to kernel
    // -------------------------------
    unsafe {
        jump_to_kernel(KERNEL_PTR, KERNEL_LEN, &mut boot_info);
    }
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}






