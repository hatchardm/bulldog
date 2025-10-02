#![no_std]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;
extern crate rlibc;

use x86_64::instructions::port::Port;
use crate::interrupts::LAPIC_TIMER_VECTOR;

pub mod framebuffer;
pub mod interrupts;
pub mod gdt;
pub mod allocator;
pub mod memory;
pub mod task;
pub mod stack;
pub mod apic;
use crate::memory::init_offset_page_table;
use crate::memory::map_lapic_mmio;
use x86_64::structures::paging::PageTableFlags;
use crate::apic::apic::init as setup_apic;
use crate::memory::BootInfoFrameAllocator;
use bootloader_api::BootInfo;
use x86_64::structures::paging::FrameAllocator;
use x86_64::structures::paging::mapper::Mapper;
use bootloader_api::info::MemoryRegion;
use x86_64::{VirtAddr, structures::paging::Size4KiB, structures::paging::mapper::MapToError};



pub fn init(
    memory_regions: &'static [MemoryRegion],
    physical_memory_offset: VirtAddr,
) -> Result<(), MapToError<Size4KiB>> {
    use crate::{gdt, interrupts, memory, stack};
    use x86_64::structures::paging::{Page, PageTableFlags};

    gdt::init();
    interrupts::init_idt();

    let mut mapper = unsafe { memory::init_offset_page_table(physical_memory_offset) };
    let mut frame_allocator = unsafe { memory::BootInfoFrameAllocator::init(memory_regions) };

    // Map LAPIC MMIO
    memory::map_lapic_mmio(&mut mapper, &mut frame_allocator);


    // Map LAPIC IST stack
    let lapic_stack_start = VirtAddr::from_ptr(unsafe { core::ptr::addr_of!(stack::LAPIC_STACK.0) });
    let lapic_stack_end = lapic_stack_start + gdt::STACK_SIZE;
    let lapic_stack_range = Page::range_inclusive(
        Page::containing_address(lapic_stack_start),
        Page::containing_address(lapic_stack_end - 1u64),
    );

    for page in lapic_stack_range {
        let frame = frame_allocator
            .allocate_frame()
            .expect("Failed to allocate frame for LAPIC stack");
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
        unsafe {
            mapper.map_to(page, frame, flags, &mut frame_allocator)?.flush();
        }
    }

    crate::setup_apic();
    x86_64::instructions::interrupts::enable();
    Ok(())
}


pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}







#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    println!("PANIC: allocation error — size: {}, align: {}", layout.size(), layout.align());
    panic!("allocation error: {:?}", layout)
}