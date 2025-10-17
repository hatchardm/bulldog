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

use x86_64::instructions::port::Port;
use crate::interrupts::LAPIC_TIMER_VECTOR;
use x86_64::PhysAddr;
use alloc::vec::Vec;
use crate::memory::PreHeapAllocator;
use x86_64::structures::paging::{FrameAllocator, PhysFrame};
use crate::memory::init_offset_page_table;
use crate::memory::map_lapic_mmio;
use x86_64::structures::paging::PageTableFlags;
use crate::apic::apic::init as setup_apic;
use crate::memory::BootInfoFrameAllocator;
use bootloader_api::BootInfo;
use x86_64::structures::paging::mapper::Mapper;
use bootloader_api::info::MemoryRegion;
use x86_64::{VirtAddr, structures::paging::Size4KiB, structures::paging::mapper::MapToError};
use core::ptr;
use crate::allocator::ALLOCATOR;
use crate::memory::find_unused_frame;
use x86_64::structures::paging::Translate;



pub mod framebuffer;
pub mod interrupts;
pub mod gdt;
pub mod allocator;
pub mod memory;
pub mod task;
pub mod stack;
pub mod apic;



pub fn init(
    memory_regions: &[MemoryRegion],
    phys_mem_offset: VirtAddr,
) -> Result<(), MapToError<Size4KiB>> {
    use crate::{gdt, interrupts, memory, stack};
    use x86_64::structures::paging::{Page, PageTableFlags};

    println!("Creating mapper");
    let mut mapper = unsafe { memory::init_offset_page_table(phys_mem_offset) };

    println!("Transmuting memory_regions to 'static");
    let memory_regions: &'static [MemoryRegion] = unsafe {
        core::mem::transmute::<&[MemoryRegion], &'static [MemoryRegion]>(memory_regions)
    };

    println!("Creating pre-heap frame allocator");
    let (temp_frames, memory_map) = unsafe {
        memory::BootInfoFrameAllocator::init_temp(memory_regions)
    };

    let mut temp_allocator = PreHeapAllocator {
    memory_map,
    frames: temp_frames,
    next: 0,
};

println!("About to call init_heap");

allocator::init_heap(&mut mapper, &mut temp_allocator)
    .expect("Heap initialization failed");

println!("Heap initialized");


println!("Creating frame allocator");
println!("Creating frame allocator");
println!("Creating frame allocator");
println!("Creating frame allocator");

let frames = temp_allocator.into_vec();

println!("About to call BootInfoFrameAllocator::new");
let mut frame_allocator = BootInfoFrameAllocator::new(memory_map, frames);
println!("BootInfoFrameAllocator::new returned");




  // println!("Logging memory regions");
   // for region in memory_regions.iter() {
   //     let virt = VirtAddr::new(region.start + phys_mem_offset.as_u64());
    //    println!(
    //         "Region: start={:#x}, virt={:#x}, type={:?}",
    //        region.start, virt, region.kind
     //   );
   // }

    gdt::init();
    interrupts::init_idt();

    println!("Mapping LAPIC MMIO");
    memory::map_lapic_mmio(&mut mapper, &mut frame_allocator);

   println!("Mapping LAPIC IST stack");

let lapic_stack_start = VirtAddr::from_ptr(unsafe { core::ptr::addr_of!(stack::LAPIC_STACK.0) });
let lapic_stack_end = lapic_stack_start + gdt::STACK_SIZE;
let lapic_stack_range = Page::range_inclusive(
    Page::containing_address(lapic_stack_start),
    Page::containing_address(lapic_stack_end - 1u64),
);






//Sync allocator with memory map before mapping LAPIC stack
frame_allocator.mark_used_frames();

//let lapic_stack_range_start = 0x2e000;
//let lapic_stack_range_end = 0x40000;

//for frame in frame_allocator.allocated.iter_used_frames() {
 //   println!("Used frame: {:#x}", frame.start_address().as_u64());
//}

println!("Starting LAPIC stack mapping loop");

// Pre-mark LAPIC stack frames to avoid allocator reuse
for page in lapic_stack_range.clone() {
    let phys = page.start_address().as_u64(); // Already physical
    let frame = PhysFrame::containing_address(PhysAddr::new(phys));
    frame_allocator.allocated.mark_used(frame);
}


println!("LAPIC stack range: virt={:#x} - {:#x}", lapic_stack_start, lapic_stack_end);
for page in lapic_stack_range.clone() {
    println!("LAPIC stack page: virt={:#x}", page.start_address());
}

// Now map each LAPIC stack page using the pre-marked frames
for page in lapic_stack_range {
    let phys = page.start_address().as_u64();
    let frame = PhysFrame::containing_address(PhysAddr::new(phys));


    let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;

    if mapper.translate_addr(page.start_address()).is_some() {
    println!("Page already mapped: {:?}", page.start_address());
    continue; // or return Err if you want to fail fast
}

println!(
    "Mapping LAPIC stack page: virt={:#x}, phys={:#x}",
    page.start_address().as_u64(),
    frame.start_address().as_u64()
);

    unsafe {
        mapper.map_to(page, frame, flags, &mut frame_allocator)?.flush();
    }
}





// Duplicate LAPIC stack mapping loop — safely commented out for clarity and performance
//for page in lapic_stack_range {
 //  let frame = find_unused_frame(&frame_allocator.allocated);

//frame.map(|f| {
   //       println!(
   //     "Mapping LAPIC stack page: virt={:?}, phys={:?}",
    //    page.start_address(),
   //     f.start_address()
  //  );
//});

    

   // frame_allocator.allocated.mark_used(frame.expect("LAPIC stack frame must be present"));



  //  println!(
    //    "Mapping LAPIC stack page: virt={:#x}, phys={:#x}",
    //    page.start_address(),
    //    frame.expect("LAPIC stack frame must be present").start_address()

   // );

  //  let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
  //  unsafe {
    //    mapper.map_to(page, frame.expect("LAPIC stack frame must be present"), flags, &mut frame_allocator)?.flush();

  //  }
//}

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