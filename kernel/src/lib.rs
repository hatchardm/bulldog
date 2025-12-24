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
use bootloader_api::info::MemoryRegion;
use log::{info, debug, error};
use x86_64::{
    PhysAddr, VirtAddr,
    structures::paging::{
        mapper::MapToError, mapper::Mapper, FrameAllocator, Page, PageTableFlags, PhysFrame, Size4KiB, Translate,
    },
};
 use crate::syscall::fd::init_fd_table_with_std;

#[macro_use]
pub mod macros;
pub mod writer;
pub mod framebuffer;
pub mod interrupts;
pub mod gdt;
pub mod allocator;
pub mod memory;
pub mod stack;
pub mod apic;
pub mod time;
pub mod font;
pub mod color;
pub mod logger;
pub mod syscall;
pub mod user_sys;
pub mod serial;

#[cfg(feature = "syscall_tests")]
mod tests;

use crate::allocator::ALLOCATOR;
use crate::apic::{lapic_read, LapicRegister, setup_apic};
use crate::memory::{BootInfoFrameAllocator, PreHeapAllocator, init_offset_page_table, map_lapic_mmio};

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
pub fn kernel_init(
    memory_regions: &'static [MemoryRegion],
    phys_mem_offset: VirtAddr,
) -> Result<(), MapToError<Size4KiB>> {
    use crate::{gdt, interrupts, memory, stack};

    disable_pic();
    #[cfg(not(feature = "syscall_tests"))]
    {info!("Creating mapper");}
    let mut mapper = unsafe { init_offset_page_table(phys_mem_offset) };

    // Log memory regions directly
    for region in memory_regions.iter() {
        debug!(
            "Region: start={:#x}, end={:#x}, kind={:?}",
            region.start, region.end, region.kind,
        );
    }

    #[cfg(not(feature = "syscall_tests"))]
    {info!("Creating pre-heap frame allocator");}
    let (temp_frames, memory_map) = unsafe { BootInfoFrameAllocator::init_temp(memory_regions) };

    let mut temp_allocator = PreHeapAllocator {
        memory_map,
        frames: temp_frames,
        next: 0,
    };

    #[cfg(not(feature = "syscall_tests"))]
    {info!("Initializing heap");}
    allocator::init_heap(&mut mapper, &mut temp_allocator).expect("Heap initialization failed");
    #[cfg(not(feature = "syscall_tests"))]
    #[cfg(not(feature = "syscall_tests"))]
    {info!("Heap initialized");}

    #[cfg(not(feature = "syscall_tests"))]
    {info!("Finalizing frame allocator from temp allocator");}
    let frames = temp_allocator.into_vec();
    let mut frame_allocator = BootInfoFrameAllocator::new(memory_map, frames);
    #[cfg(not(feature = "syscall_tests"))]
    {info!("Frame allocator ready");}

    // Optional: identity-map framebuffer region here if needed

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

     // 🧩 Register syscall handler BEFORE enabling interrupts
    crate::syscall::init_syscall();
    #[cfg(not(feature = "syscall_tests"))]
    {info!("Syscall handler ready");}

    init_fd_table_with_std();

   #[cfg(feature = "syscall_tests")]
   tests::syscall_harness::run_syscall_tests();
   


    // APIC MMIO mapping
    #[cfg(not(feature = "syscall_tests"))]
    {info!("Mapping LAPIC MMIO");}
    map_lapic_mmio(&mut mapper, &mut frame_allocator);

    // APIC IST stack mapping
    #[cfg(not(feature = "syscall_tests"))]
    {info!("Mapping LAPIC IST stack");}
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

    #[cfg(not(feature = "syscall_tests"))]
    {info!("Pre-mark LAPIC stack frames");}
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
    #[cfg(not(feature = "syscall_tests"))]
    {info!("LAPIC CURRENT COUNT: {}", count);}

    #[cfg(not(feature = "syscall_tests"))]
    {info!("Enabling interrupts");}
    x86_64::instructions::interrupts::enable();
    #[cfg(not(feature = "syscall_tests"))]
    {info!("Exiting init");}

    Ok(())
}

/// Disable legacy PIC by masking all IRQs.
/// Ensures APIC is the sole interrupt controller.
pub fn disable_pic() {
    unsafe {
        let mut pic1 = x86_64::instructions::port::Port::new(0x21);
        let mut pic2 = x86_64::instructions::port::Port::new(0xA1);
        pic1.write(0xFFu8);
        pic2.write(0xFFu8);
    }
}

/// Allocator error handler.
/// Logs and panics on allocation failure.
#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    error!("PANIC: allocation error — size: {}, align: {}", layout.size(), layout.align());
    panic!("allocation error: {:?}", layout)
}

/// Halt loop: the kernel’s idle routine.
/// 
/// - Puts the CPU into a low‑power state (`hlt`) until the next interrupt.
/// - Uses a watchdog to detect stalls in the tick counter.
/// - Runs a periodic health check to log kernel liveness.
/// 
/// Safety: must only be called once interrupts and the LAPIC timer are configured.
/// Otherwise the CPU will halt indefinitely without waking.
pub fn hlt_loop() -> ! {
    let mut wd = crate::time::Watchdog::new(5000u64, 3u32, 2u32);

    loop {
        unsafe { core::arch::asm!("hlt"); }
        wd.check();

        // Only run health checks if not in syscall_tests mode
        #[cfg(not(feature = "syscall_tests"))]
        crate::time::health_check(1000);
    }
}





