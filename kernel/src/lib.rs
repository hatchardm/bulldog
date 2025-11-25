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
use log::{info, debug, warn, error, trace};
use x86_64::{
    PhysAddr, VirtAddr,
    structures::paging::{
        mapper::MapToError, mapper::Mapper, FrameAllocator, Page, PageTableFlags, PhysFrame, Size4KiB, Translate,
    },
};

#[macro_use]
pub mod macros;
pub mod writer;
pub mod framebuffer;
pub mod interrupts;
pub mod gdt;
pub mod allocator;
pub mod memory;
pub mod task;
pub mod stack;
pub mod apic;
pub mod time;
pub mod font;
pub mod color;
pub mod logger;
use crate::allocator::ALLOCATOR;
use crate::apic::{lapic_read, LapicRegister};
use crate::apic::setup_apic;
use crate::memory::{BootInfoFrameAllocator, PreHeapAllocator};
use crate::memory::{init_offset_page_table, map_lapic_mmio};


pub fn kernel_init(
    memory_regions: &'static [MemoryRegion],
    phys_mem_offset: VirtAddr,
) -> Result<(), MapToError<Size4KiB>> {
    use crate::{gdt, interrupts, memory, stack};

    disable_pic();

    info!("Creating mapper");
    let mut mapper = unsafe { init_offset_page_table(phys_mem_offset) };

    // Log memory regions directly without transmuting to 'static
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

    info!("Finalizing frame allocator from temp allocator");
    let frames = temp_allocator.into_vec();
    let mut frame_allocator = BootInfoFrameAllocator::new(memory_map, frames);
    info!("Frame allocator ready");

    // Optional: identity-map framebuffer region here if needed
    // If WRITER’s framebuffer becomes invalid post-paging, pass the framebuffer phys range into this
    // function and identity-map it:
    // memory::identity_map_framebuffer(&mut mapper, &mut frame_allocator, fb_phys_start, fb_len_bytes)?;

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

    // Sync allocator state before marking frames
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
        // Translate the current mapping, if any
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
                // If unmapped, you need a physical frame to back this page.
                // Choose an appropriate frame source for the LAPIC stack.
                let phys = page.start_address(); // This is a VIRT address; you must provide a real PhysFrame.
                error!(
                    "No translation for LAPIC stack page {:?}; supply a backing PhysFrame from allocator",
                    phys
                );
                // If you intend identity mapping for the stack region, compute its physical range and use that:
                // let frame = PhysFrame::containing_address(PhysAddr::new(desired_phys_addr));
                // unsafe { mapper.map_to(page, frame, PageTableFlags::PRESENT | PageTableFlags::WRITABLE, &mut frame_allocator)?.flush(); }
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



pub fn disable_pic() {
    unsafe {
        let mut pic1 = x86_64::instructions::port::Port::new(0x21);
        let mut pic2 = x86_64::instructions::port::Port::new(0xA1);
        pic1.write(0xFFu8); // Mask all IRQs on PIC1
        pic2.write(0xFFu8); // Mask all IRQs on PIC2
    }
}

#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    error!("PANIC: allocation error — size: {}, align: {}", layout.size(), layout.align());
    panic!("allocation error: {:?}", layout)
}



pub fn hlt_loop() -> ! {
    // Tune to your tick rate and expected responsiveness.
    let mut wd = crate::time::Watchdog::new(5000u64, 3u32, 2u32);

    loop {
        // Sleep until next interrupt (timer tick should increment TICKS).
        unsafe { core::arch::asm!("hlt"); }

        // Silent unless stall persists.
        wd.check();

        // Single heartbeat source.
        crate::time::health_check(1000);
    }
}



