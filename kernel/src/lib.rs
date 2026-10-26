//! Bulldog kernel crate root.
//!
//! - `#![no_std]`: no standard library, only `core`.
//! - `#![no_main]`: custom entry point defined elsewhere.
//! - Feature gates enable x86 interrupt ABI, custom test harness, and allocator error handling.
//!
//! This file wires together core subsystems (APIC, GDT, interrupts, memory, etc.)
//! and provides the kernel initialization routine and idle loop.

#![no_std]
#![allow(warnings)]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;
extern crate rlibc;

use alloc::vec::Vec;
use alloc::boxed::Box;
use boot_proto::MemoryRegion;   // ← updated
use boot_proto::BootInfo;
use log::{info, debug, error};
use x86_64::{
    PhysAddr, VirtAddr,
    structures::paging::{
        mapper::MapToError, mapper::Mapper, FrameAllocator, Page, PageTableFlags,
        PhysFrame, Size4KiB, Size1GiB, Translate,
    },
};

use crate::serial::{serial_println, serial_print, serial_print_hex_u64, serial_print_u64};
use crate::color::Color;
use crate::text::draw_char_8x16;
use crate::font8x16::font8x16;
use crate::writer::Writer;
use log::LevelFilter;
use crate::logger::logger_init;
use crate::logger::set_framebuffer_ready;
use crate::writer::WRITER;


#[macro_use]
pub mod macros;
pub mod serial;
pub mod writer;
pub mod framebuffer;
pub mod interrupts;
pub mod gdt;
pub mod allocator;
pub mod memory;
pub mod stack;
pub mod apic;
pub mod time;
pub mod color;
pub mod logger;
pub mod syscall;
pub mod vfs;
pub mod elf;
pub mod user_mode;
pub mod text;
pub mod font8x16;
pub mod paging;
pub mod fs;

#[cfg(feature = "syscall_tests")]
mod tests;

use crate::allocator::ALLOCATOR;
use crate::apic::{lapic_read, LapicRegister, setup_apic};
use crate::memory::{
    BootInfoFrameAllocator, PreHeapAllocator,
    init_offset_page_table, map_lapic_mmio
};
use crate::syscall::fd::init_fd_table_with_std;
use crate::vfs::init::vfs_init;
use crate::framebuffer::KernelFramebuffer;
use crate::framebuffer::PixelFormat;
use crate::paging::{MAPPER, FRAME_ALLOCATOR};
use crate::allocator::init_heap;
use crate::elf::types::Elf64_Ehdr;
use crate::elf::loader::{validate_elf_header, get_program_headers, load_segments};
use crate::user_mode::enter_user_mode;

static mut FAT_ADDR: u64 = 0;
static mut FAT_SIZE: u64 = 0;

pub fn fat_phys_addr() -> u64 {
    unsafe { FAT_ADDR }
}

pub fn fat_size() -> u64 {
    unsafe { FAT_SIZE }
}


#[unsafe(no_mangle)]
pub fn kernel_main(boot_info: &mut BootInfo) -> ! {
    serial_println("Entered kernel_main");
    unsafe {
    FAT_ADDR = boot_info.fat_phys_addr;
    FAT_SIZE = boot_info.fat_size;
}


    // 🎨 Framebuffer setup
    let mut fb = KernelFramebuffer::from_bootinfo(boot_info);

    let black = fb.pack_color(Color::BLACK.r, Color::BLACK.g, Color::BLACK.b);
    fb.clear_fast(black);

    // Create writer
    let mut writer = Writer::new(&mut fb);

    // Configure writer BEFORE moving it
    writer.fg = Color::WHITE;
    writer.bg = Color::BLACK;

    // Move writer into global WRITER
    {
        let mut w = WRITER.lock();
        *w = Some(writer);
    }

    set_framebuffer_ready(true);

    // Use WRITER for boot banner
    {
        let mut w = WRITER.lock();
        if let Some(w) = w.as_mut() {
            w.write_str("🐾 Bulldog Kernel Booting...\n");
            w.write_str("Framebuffer OK\n");
            w.write_str("Pixel format detected at runtime\n");
            w.write_str("Initializing subsystems...\n\n");
        }
    }

    // Logging
    logger_init(LevelFilter::Info);
    info!("Logger initialized");
    info!("Framebuffer format: {:?}, size: {}x{}", fb.pixel_format, fb.width, fb.height);

    // Prepare memory inputs for kernel_init
    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let memory_regions = unsafe {
    core::slice::from_raw_parts(
        boot_info.memory_regions,
        boot_info.memory_region_count,
    )
};


match kernel_init(memory_regions, phys_mem_offset) {
    Ok(_) => info!("kernel_init completed successfully"),
    Err(_) => error!("kernel_init failed"),
}

 hlt_loop();

  //  loop {
  //      unsafe { core::arch::asm!("hlt"); }
  //  }
}


/// Kernel initialization routine.
///
/// - Disables legacy PIC.
/// - Sets up paging and frame allocator.
/// - Initializes heap.
/// - Loads GDT and IDT.
/// - Maps LAPIC MMIO and IST stack.
/// - Configures APIC and enables interrupts.
///
/// Optional (features):
/// - `usermode_hello`  → load /bin/hello and enter ring 3
/// - `syscall_tests`   → run in-kernel syscall harness
pub fn kernel_init(
    memory_regions: &'static [MemoryRegion],
    phys_mem_offset: VirtAddr,
) -> Result<(), ()> {
    info!("kernel_init: starting");

    // Heap
    {
        let mut mapper_guard = MAPPER.lock();
        let mapper = mapper_guard.as_mut().expect("MAPPER not initialized");

        let mut fa_guard = FRAME_ALLOCATOR.lock();
        let fa = fa_guard.as_mut().expect("FRAME_ALLOCATOR not initialized");

        info!("Initializing Heap");
        init_heap(mapper, fa).expect("init_heap failed");
        info!("Heap initialized");
    }

    // FAT32 (heap ready)
    info!("Initializing FAT32 RamDisk");

    let disk = crate::fs::block::RamDisk {
        phys_addr: unsafe { FAT_ADDR },
        size: unsafe { FAT_SIZE },
    };

    info!(
        "FAT RamDisk: phys={:#x}, size={} bytes",
        disk.phys_addr, disk.size
    );

    let bpb = crate::fs::fat::FatBpb::read(&disk);
    info!(
        "BPB: bytes_per_sector={} sectors_per_cluster={} reserved_sectors={} num_fats={} fat_size_sectors={} root_cluster={}",
        bpb.bytes_per_sector,
        bpb.sectors_per_cluster,
        bpb.reserved_sectors,
        bpb.num_fats,
        bpb.fat_size_sectors,
        bpb.root_cluster
    );

    let fat = crate::fs::fat::FatTable::read(&disk, &bpb);
    info!("FAT table loaded: {} entries", fat.entries.len());

    crate::fs::fat::set_global_fat(bpb, fat);
    info!("FAT32 initialized");

    let bpb = crate::fs::fat::get_global_bpb();
    let fat = crate::fs::fat::get_global_fat();
    let disk = crate::fs::block::RamDisk {
        phys_addr: unsafe { FAT_ADDR },
        size: unsafe { FAT_SIZE },
    };

    let root_entries =
        crate::fs::fat::read_directory(&disk, bpb, fat, bpb.root_cluster);
    info!("Root directory entries:");
    for e in root_entries.iter() {
        let name = core::str::from_utf8(&e.name).unwrap_or("???");
        info!("  {}  cluster={}  size={}", name, e.cluster(), e.size);
    }

    // GDT / TSS
    info!("Initializing GDT/TSS");
    gdt::init();
    info!("GDT/TSS Initialized");

    // PIC off
    info!("Disabling PIC");
    disable_pic();
    info!("PIC Disabled");

    // IDT
    info!("Initializing IDT");
    interrupts::init_idt();
    info!("IDT Initialized");

    // Syscall handler
    info!("Initializing syscall handler");
    crate::syscall::init_syscall();
    info!("Syscall handler ready");

    // LAPIC MMIO
    {
        let mut mapper_guard = MAPPER.lock();
        let mapper = mapper_guard.as_mut().expect("MAPPER not initialized");

        let mut fa_guard = FRAME_ALLOCATOR.lock();
        let fa = fa_guard.as_mut().expect("FRAME_ALLOCATOR not initialized");

        info!("Mapping LAPIC MMIO");
        map_lapic_mmio(mapper, fa);
        info!("LAPIC MMIO mapped");
    }

    info!("Setting up APIC");
    apic::setup_apic();
    info!("APIC setup complete");

    info!("Enabling Interrupts");
    x86_64::instructions::interrupts::enable();
    info!("Interrupts Enabled");

    vfs_init();
    info!("VFS initialized");

    init_fd_table_with_std();
    info!("FD table initialized");



// ------------------------------------------------------------------
// Option 5: usermode hello
// ------------------------------------------------------------------
#[cfg(feature = "usermode_hello")]
{
    run_usermode_elf("/bin/hello", true);
}

// ------------------------------------------------------------------
// Option 6: usermode goodbye
// ------------------------------------------------------------------
#[cfg(feature = "usermode_goodbye")]
{
    run_usermode_elf("/bin/goodbye", true);
}

// ------------------------------------------------------------------
// Option 7: hello then goodbye on exit
// ------------------------------------------------------------------
#[cfg(feature = "usermode_chain")]
{
    run_usermode_elf("/bin/hello", false); // false → arm chain to goodbye
}


    // ------------------------------------------------------------------
    // Options 3 / 4: in-kernel syscall harness
    // (must be outside the usermode block)
    // ------------------------------------------------------------------
    #[cfg(feature = "syscall_tests")]
    {
        info!("Running syscall harness");
        crate::tests::syscall_harness::run_syscall_tests();
        info!("Syscall harness completed");
    }

    info!("kernel_init: completed");
    Ok(())
}

/// Load ELF from VFS, map it, arm after-exit policy, enter ring 3. Does not return.
#[cfg(any(
    feature = "usermode_hello",
    feature = "usermode_goodbye",
    feature = "usermode_chain"
))]
pub fn run_usermode_elf(path: &str, idle_after_exit: bool) -> ! {
    info!("Loading {} ELF", path);

    // Re-init FDs for this process (needed on chain after sys_exit cleared them)
    init_fd_table_with_std();
    // ensure stdout (fd 1) is wired the same way as before first enter

    let elf_bytes = match crate::vfs::read_all(path) {
        Ok(b) => {
            info!("read_all({}) -> {} bytes", path, b.len());
            b
        }
        Err(e) => {
            error!("read_all({}) failed: {:?}", path, e);
            panic!("failed to read {}", path);
        }
    };

    let hdr = unsafe { &*(elf_bytes.as_ptr() as *const Elf64_Ehdr) };
    info!("ELF ident: {:02x?}", &hdr.e_ident[..4]);

    if let Err(e) = validate_elf_header(hdr) {
        panic!("invalid ELF header: {:?}", e);
    }

    let phdrs = match get_program_headers(&elf_bytes, hdr) {
        Ok(p) => {
            info!("program headers count: {}", p.len());
            p
        }
        Err(e) => panic!("invalid program headers: {:?}", e),
    };

    info!("Mapping ELF segments...");
    match load_segments(&elf_bytes, hdr, phdrs, |vaddr, mem_size, file_bytes, flags| {
        info!(
            "segment: vaddr={:#x} mem_size={:#x} file_len={}",
            vaddr, mem_size, file_bytes.len()
        );
        let r = crate::memory::map_user_segment(vaddr, mem_size, file_bytes, flags);
        match &r {
            Ok(()) => info!("segment mapped ok"),
            Err(()) => info!("segment map FAILED"),
        }
        r
    }) {
        Ok(()) => info!("All segments mapped"),
        Err(e) => {
            error!("load_segments failed: {:?}", e);
            panic!("segment mapping failed: {:?}", e);
        }
    }

    let user_stack_top = match crate::memory::alloc_user_stack() {
        Ok(s) => {
            info!("user stack top: {:#x}", s);
            s
        }
        Err(_) => panic!("failed to allocate user stack"),
    };

    let entry = hdr.e_entry;
    info!("Entering user mode at entry={:#x}", entry);

    if idle_after_exit {
        crate::syscall::exit::arm_exit_to_idle();
    } else {
        crate::syscall::exit::arm_exit_chain_goodbye();
    }

    unsafe {
        crate::memory::force_user_access(
            VirtAddr::new(entry),
            0xffff_8000_0000_0000,
        );
        enter_user_mode(entry, user_stack_top);
    }
}

pub fn kernel_idle_after_exit() -> ! {
    info!("back in kernel idle after process exit");
    loop {
        unsafe {
            core::arch::asm!("sti; hlt", options(nomem, nostack));
        }
    }
}

/// Disable legacy PIC by masking all IRQs.
pub fn disable_pic() {
    unsafe {
        let mut pic1 = x86_64::instructions::port::Port::new(0x21);
        let mut pic2 = x86_64::instructions::port::Port::new(0xA1);
        pic1.write(0xFFu8);
        pic2.write(0xFFu8);
    }
}

/// Allocator error handler.
#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    error!("PANIC: allocation error — size: {}, align: {}", layout.size(), layout.align());
    panic!("allocation error: {:?}", layout)
}

/// Kernel idle loop.
pub fn hlt_loop() -> ! {
    let mut wd = crate::time::Watchdog::new(5000u64, 3u32, 2u32);

    loop {
        unsafe { core::arch::asm!("hlt"); }
        wd.check();

        #[cfg(not(feature = "syscall_tests"))]
        crate::time::health_check(1000);
    }
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    // Serial
    unsafe {
        serial_print("[PANIC] ");
        if let Some(location) = info.location() {
            serial_print("at ");
            serial_print(location.file());
            serial_print(":");
            serial_print_hex_u64(location.line() as u64);
        }
        serial_print("\n");
    }

    // Framebuffer
    if crate::logger::FB_READY.load(core::sync::atomic::Ordering::Relaxed) {
        let mut guard = crate::writer::WRITER.lock();
        if let Some(w) = guard.as_mut() {
            w.set_log_level_color(crate::writer::LogLevel::Error);
            w.write_str("[PANIC] ");
            if let Some(location) = info.location() {
                use core::fmt::Write;
                let _ = write!(w, "at {}:{}\n", location.file(), location.line());
            } else {
                w.write_str("no location info\n");
            }
        }
    }

    loop {
        unsafe { core::arch::asm!("hlt"); }
    }
}






