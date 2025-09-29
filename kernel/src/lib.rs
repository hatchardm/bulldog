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

pub mod framebuffer;
pub mod interrupts;
pub mod gdt;
pub mod allocator;
pub mod memory;
pub mod task;
pub mod stack;
pub mod apic;
use crate::apic::apic::init as setup_apic;


pub fn init() {
    
    gdt::init();
    interrupts::init_idt();
    init_pit();
    setup_apic();
    x86_64::instructions::interrupts::enable();
}

pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

pub fn init_pit() {
    let mut command = Port::new(0x43);
    let mut channel0 = Port::new(0x40);
    unsafe {
        command.write(0x36u8); // Set PIT to mode 3 (square wave)
        let divisor: u16 = (1193182u32 / 100) as u16; // 100Hz
        channel0.write((divisor & 0xFF) as u8); // low byte
        channel0.write((divisor >> 8) as u8);   // high byte
    }
}





#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("allocation error: {:?}", layout)
}