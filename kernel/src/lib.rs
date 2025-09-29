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
use x86_64::VirtAddr;
use crate::apic::apic::init as setup_apic;



pub fn init() {
    
    gdt::init();
    interrupts::init_idt();
    let phys_mem_offset = VirtAddr::new(0xFFFF800000000000); // adjust to Bulldog’s actual offset
    let mut mapper = unsafe { init_offset_page_table(phys_mem_offset) };
    map_lapic_mmio(&mut mapper);
    setup_apic();

    

    x86_64::instructions::interrupts::enable();
}

pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}







#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("allocation error: {:?}", layout)
}