#![no_std]
#![no_main]


extern crate alloc;

use bootloader_api::entry_point;
use bootloader_api::config::{BootloaderConfig, Mapping};
use core::panic::PanicInfo;


//If you used optional features, such as map-physical-memory, you can enable them again through the entry_point macro:

  const CONFIG: BootloaderConfig = {
  let mut config = BootloaderConfig::new_default();
  config.kernel_stack_size = 100 * 1024; // 100 KiB
  config.mappings.physical_memory = Some(Mapping::Dynamic);
  config
};

    use kernel::{print, println};
    use kernel::allocator;
    use kernel::memory::{self, BootInfoFrameAllocator};
    use x86_64::VirtAddr;
    use kernel::init;
    use kernel::task::{executor::Executor, keyboard, Task};
    use kernel::framebuffer;
    use kernel::interrupts::PICS;
    use kernel::gdt;
    use crate::gdt::STACK_SIZE;
   
   
entry_point!(kernel_main, config = &CONFIG);


fn kernel_main(boot_info: &'static mut bootloader_api::info::BootInfo) -> ! {
    use kernel::gdt::STACK_SIZE;
    use kernel::stack::STACK;
    use x86_64::VirtAddr;
    use x86_64::structures::paging::FrameAllocator;
    use x86_64::structures::paging::{Page, PageTableFlags, Mapper};
    use x86_64::structures::paging::mapper::MapToError;
    use kernel::stack::get_stack_start;



    // Initialize framebuffer
    framebuffer::init(boot_info.framebuffer.as_mut().unwrap());

    println!(" ");
    println!("LOADING BULLDOG");
    println!("   ");

    // Unmask timer and keyboard interrupts
    unsafe {
        PICS.lock().initialize();
        PICS.lock().write_masks(0b1111_1100, 0b1111_1111);
    }

    // Initialize paging and frame allocator
    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset.into_option().unwrap());
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_regions) };

    let stack_start = get_stack_start();
    let stack_end = stack_start + STACK_SIZE;

    



for page in Page::range_inclusive(
    Page::containing_address(stack_start),
    Page::containing_address(stack_end - 1u64),
) {
    let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;

    // Check if the page is already mapped
    if mapper.translate_page(page).is_ok() {
        // Skip already mapped pages
        continue;
    }

    let frame = frame_allocator
        .allocate_frame()
        .expect("No usable frame for stack");

    unsafe {
        mapper
            .map_to(page, frame, flags, &mut frame_allocator)
            .expect("Stack mapping failed")
            .flush();
    }
}




    
    // Initialize heap
    println!("Executing init_heap");
    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap initialization failed");
    println!("I have gone past the allocator init_heap");

    // Load GDT, TSS, and IDT
    init();
    println!("INTERUPTS INITIATED");
    println!("  ");


   // println!("Triggering stack overflow...");
   // trigger_stack_overflow();  //Tests stack overflow fault and Double Fault

   
    //x86_64::instructions::interrupts::int3();


     let mut executor = Executor::new();
      executor.spawn(Task::new(example_task()));
      executor.spawn(Task::new(keyboard::print_keypresses()));
      executor.run();


  //  println!("It did not crash");

  //  loop {}
}

    
    
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    unsafe {
        framebuffer::WRITER.force_unlock();
    };
    println!("{}", info);
    kernel::hlt_loop();
}

//#[cfg(test)]
//#[panic_handler]
//fn panic(info: &PanicInfo) -> ! {
   // kernel::test_panic_handler(info)
//}






async fn async_number() -> u32 {
  42
}

async fn example_task() {
  let number = async_number().await;
  println!("async number: {}", number);
}
 
#[allow(unconditional_recursion)]
fn trigger_stack_overflow() {
    trigger_stack_overflow();
    let _ = unsafe { core::ptr::read_volatile(&0u8) };

   // prevent tail call optimization
}


