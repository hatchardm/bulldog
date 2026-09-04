#![no_main]
#![no_std]

extern crate alloc;

use uefi::prelude::*;
use uefi::boot as uefi_boot;
use uefi::proto::console::text::Input;
use uefi::proto::console::gop::PixelFormat as UefiPixelFormat;
use uefi::boot::MemoryType;

use log::info;

mod gop;
mod framebuffer;
mod console;
mod color;
mod text;
mod boot;
mod paging;

use gop::init as init_graphics;
use text::load_font;
use console::Console;
use color::Color;
use paging::setup_minimal_paging;
use paging::HHDM_BASE;

use boot::{
    load_kernel,
    jump_to_kernel,
    find_rsdp,
    acpi_revision,
    fill_memory_regions_from_map,
};

use boot_proto::{
    BootInfo,
    Framebuffer,
    PixelFormat,
    MemoryRegion,
    MemoryRegionKind,
    MAX_MEMORY_REGIONS,
};

const PHYS_MEM_OFFSET: u64 = HHDM_BASE;

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

    let gop_mode = ctx.gop.current_mode_info();

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
// Allocate BootInfo in LOADER_DATA
// -------------------------------
let boot_info_pool = uefi_boot::allocate_pool(
    MemoryType::LOADER_DATA,
    core::mem::size_of::<BootInfo>(),
).expect("Failed to allocate BootInfo");

let boot_info: &mut BootInfo = unsafe {
    &mut *(boot_info_pool.as_ptr() as *mut BootInfo)
};

// -------------------------------
// Build Framebuffer (correct location)
// -------------------------------
let proto_pf = match gop_mode.pixel_format() {
    UefiPixelFormat::Rgb      => PixelFormat::Rgb,
    UefiPixelFormat::Bgr      => PixelFormat::Bgr,
    UefiPixelFormat::Bitmask  => PixelFormat::Bitmask,
    UefiPixelFormat::BltOnly  => PixelFormat::BltOnly,
};

let framebuffer = Framebuffer {
    addr: fb_addr_phys,
    width: fb_width as u32,
    height: fb_height as u32,
    stride: fb_stride as u32,
    pixel_format: proto_pf,
};

info!("Framebuffer phys addr: {:#x}", fb_addr_phys);

// -------------------------------
// Initialize BootInfo (single clean initialization)
// -------------------------------
*boot_info = BootInfo {
    framebuffer,
    framebuffer_present: 1,
    _pad0: [0; 7],
    physical_memory_offset: PHYS_MEM_OFFSET,
    memory_regions: core::ptr::null(),
    memory_region_count: 0,
    memory_regions_buffer: [MemoryRegion {
        start: 0,
        end: 0,
        kind: MemoryRegionKind::Reserved,
    }; MAX_MEMORY_REGIONS],
    kernel_phys_start: 0,
    kernel_phys_end: 0,
    kernel_entry_phys: 0,
    framebuffer_virt: PHYS_MEM_OFFSET + fb_addr_phys,
};


    // -------------------------------
    // Exit Boot Services
    // -------------------------------


    info!("about to call ExitBootServices");

    let memory_map_owned = unsafe { uefi_boot::exit_boot_services(None) };

    info!("ExitBootServices returned");

    

    fill_memory_regions_from_map(boot_info, &memory_map_owned);






    // Paging + HHDM
    

    setup_minimal_paging(&memory_map_owned);

    



    // -------------------------------
    // Jump to kernel with safe BootInfo
    // -------------------------------
    unsafe {
        jump_to_kernel(KERNEL_PTR, KERNEL_LEN, boot_info);
    }
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}




