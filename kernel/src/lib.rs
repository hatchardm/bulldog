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
        PhysFrame, Size4KiB, Translate,
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




#[unsafe(no_mangle)]
pub fn kernel_main(boot_info: &mut BootInfo) -> ! {
    serial_println("Entered kernel_main");

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
/// Returns `Ok(())` if initialization succeeds, or a `MapToError` if paging fails.
/// 
 
pub fn kernel_init(
    memory_regions: &'static [MemoryRegion],
    phys_mem_offset: VirtAddr,
) -> Result<(), ()> {
    info!("kernel_init: starting");

    info!("Memory map from bootloader:");
    for region in memory_regions {
        info!(
            "  kind={:?}, start={:#018x}, end={:#018x}",
            region.kind,
            region.start,
            region.end,
        );
    }

    info!("Physical memory offset (HHDM): {:#018x}", phys_mem_offset.as_u64());

    // 🔹 Use global mapper + frame allocator (unsafe because of static mut)
    let mut mapper_guard = MAPPER.lock();
    let mapper = mapper_guard.as_mut().expect("MAPPER not initialized");

    let mut fa_guard = FRAME_ALLOCATOR.lock();
    let fa = fa_guard.as_mut().expect("FRAME_ALLOCATOR not initialized");
    info!("Initializing Heap");
    init_heap(mapper, fa).expect("init_heap failed");
    info!("Heap initialized");

    info!("Initializing GDT/TSS");
    gdt::init();
    info!("GDT/TSS Initialized");

    info!("Disabling PIC");
    disable_pic();
    info!("PIC Disabled");

    info!("Initializing IDT");
    interrupts::init_idt();
    info!("IDT Initialized");

    info!("Mapping LAPIC MMIO");
    map_lapic_mmio(mapper, fa);
    info!("LAPIC MMIO mapped");

    info!("Setting up APIC");
    apic::setup_apic();
    info!("APIC setup complete");

    info!("Enabling Interrupts");
    x86_64::instructions::interrupts::enable();
    info!("Interrupts Enabled");


    info!("kernel_init: completed");
    Ok(())
}





 
/* 
pub fn kernel_init(
    memory_regions: &'static [MemoryRegion],   // ← updated
    phys_mem_offset: VirtAddr,
) -> Result<(), MapToError<Size4KiB>> {
    use crate::{gdt, interrupts, memory, stack};

    disable_pic();
    info!("Creating mapper");
    let mut mapper = unsafe { init_offset_page_table(phys_mem_offset) };

    // Log memory regions directly
    for region in memory_regions.iter() {
        debug!(
            "Region: start={:#x}, end={:#x}, kind={:?}",
            region.start, region.end, region.kind,
        );
    }

    info!("Creating pre-heap frame allocator");
    let (temp_frames, memory_map) = unsafe { BootInfoFrameAllocator::init_temp(memory_regions) };

    let mut temp_allocator = PreHeapAllocator {
        memory_map,
        frames: temp_frames,
        next: 0,
    };

    info!("Initializing heap");
    allocator::init_heap(&mut mapper, &mut temp_allocator).expect("Heap initialization failed");
    info!("Heap initialized");

    init_fd_table_with_std();

    info!("Finalizing frame allocator from temp allocator");
    let frames = temp_allocator.into_vec();
    let mut frame_allocator = BootInfoFrameAllocator::new(memory_map, frames);
    info!("Frame allocator ready");

    debug!("Logging memory regions with virt addresses");
    for region in memory_regions.iter() {
        let virt = VirtAddr::new(region.start + phys_mem_offset.as_u64());
        debug!(
            "Region: start={:#x}, virt={:#x}, type={:?}",
            region.start, virt, region.kind
        );
    }

    // Core CPU tables
    gdt::init();
    interrupts::init_idt();

    // Syscall handler
    crate::syscall::init_syscall();
    info!("Syscall handler ready");

    #[cfg(feature = "syscall_tests")]
    {
        vfs_init();
        tests::syscall_harness::run_syscall_tests();
    }

    // APIC MMIO mapping
    info!("Mapping LAPIC MMIO");
    map_lapic_mmio(&mut mapper, &mut frame_allocator);

    // APIC IST stack mapping
    info!("Mapping LAPIC IST stack");
    let lapic_stack_start = VirtAddr::from_ptr(unsafe { core::ptr::addr_of!(stack::LAPIC_STACK.0) });
    let lapic_stack_end = lapic_stack_start + gdt::STACK_SIZE;
    let lapic_stack_range = Page::range_inclusive(
        Page::containing_address(lapic_stack_start),
        Page::containing_address(lapic_stack_end - 1u64),
    );

    frame_allocator.mark_used_frames();

    for frame in frame_allocator.allocated.iter_used_frames() {
        debug!("Used frame: {:#x}", frame.start_address().as_u64());
    }

    info!("Pre-mark LAPIC stack frames");
    for page in lapic_stack_range.clone() {
        if let Some(phys) = mapper.translate_addr(page.start_address()) {
            let frame = PhysFrame::containing_address(phys);
            frame_allocator.allocated.mark_used(frame);
        } else {
            error!("LAPIC stack page not mapped: {:?}", page.start_address());
        }
    }

    debug!("LAPIC stack range: virt={:#x} - {:#x}", lapic_stack_start, lapic_stack_end);
    for page in lapic_stack_range {
        match mapper.translate_addr(page.start_address()) {
            Some(phys) => {
                let frame = PhysFrame::containing_address(phys);
                let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;

                debug!("Ensure flags: remapping page {:?}", page.start_address());
                unsafe {
                    mapper.unmap(page)
                        .map_err(|_| MapToError::FrameAllocationFailed)?
                        .1
                        .flush();

                    mapper.map_to(page, frame, flags, &mut frame_allocator)?.flush();
                }
            }
            None => {
                error!(
                    "No translation for LAPIC stack page {:?}; supply a backing PhysFrame from allocator",
                    page.start_address()
                );
            }
        }
    }

    setup_apic();

    let count = lapic_read(LapicRegister::CURRENT_COUNT);
    info!("LAPIC CURRENT COUNT: {}", count);

    info!("Enabling interrupts");
    x86_64::instructions::interrupts::enable();
    info!("Exiting init");

    Ok(())
}
*/

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






