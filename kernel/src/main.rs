#![no_std]
#![no_main]

extern crate alloc;

use bootloader_api::{
    config::{BootloaderConfig, Mapping},
    entry_point,
    info::BootInfo,
};
use core::panic::PanicInfo;
use x86_64::{structures::paging::PageTableFlags, VirtAddr};

use kernel::{
    allocator,
    framebuffer,
    gdt::{self, STACK_SIZE},
    hlt_loop,
    memory::{self, BootInfoFrameAllocator},
    stack,
    task::{executor::Executor, keyboard, Task},
    interrupts, // assuming this module exists
    println,
};

use x86_64::structures::paging::mapper::Translate;
use x86_64::structures::paging::FrameAllocator;


// 🛠 Bootloader configuration
use bootloader_api::config:: FrameBuffer;

const CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();
    config.kernel_stack_size = 100 * 1024; // 100 KiB
    config.mappings.physical_memory = Some(Mapping::Dynamic);
    config
};




entry_point!(kernel_main, config = &CONFIG);

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    // ✅ Framebuffer first—enables println!()
    let framebuffer = boot_info.framebuffer.as_mut().unwrap();

// Extract info before init to avoid borrow conflict
let fb_info = framebuffer.info();
let fb_ptr = framebuffer.buffer().as_ptr() as usize;

framebuffer::init(framebuffer); // Must come before println! so output works
println!("LOADING BULLDOG KERNEL");
println!("Framebuffer physical address: {:#x}", fb_ptr);
println!("Framebuffer range: {:#x} - {:#x}", fb_ptr, fb_ptr + fb_info.byte_len);






    // ✅ Extract memory info early to avoid borrow conflicts
    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset.into_option().unwrap());
    let memory_regions = &boot_info.memory_regions;


    // ✅ Initialize paging and frame allocator
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe {
    BootInfoFrameAllocator::init(&*boot_info.memory_regions)
};


    // ✅ Map LAPIC IST stack
    let stack_start = stack::get_stack_start();
    let stack_end = stack_start + STACK_SIZE;


   for page in x86_64::structures::paging::Page::range_inclusive(
    x86_64::structures::paging::Page::containing_address(stack_start),
    x86_64::structures::paging::Page::containing_address(stack_end - 1u64),
) {
    let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
    let virt = page.start_address();

    if mapper.translate_addr(virt).is_some() {
        println!("Page mapped: {:?}", virt);
    } else {
        println!("Page NOT mapped: {:?}", virt);
    }

    let frame = frame_allocator
        .allocate_frame()
        .expect("No usable frame for LAPIC stack");

    unsafe {
        use x86_64::structures::paging::mapper::Mapper;
        mapper.map_to(page, frame, flags, &mut frame_allocator)
            .expect("Stack mapping failed")
            .flush();
    }




        let frame = frame_allocator
            .allocate_frame()
            .expect("No usable frame for LAPIC stack");

        unsafe {
            use x86_64::structures::paging::mapper::Mapper;
            mapper.map_to(page, frame, flags, &mut frame_allocator)
                .expect("Stack mapping failed")
                .flush();
        }
    }

    // ✅ Heap init
    println!("Executing init_heap");
    allocator::init_heap(&mut mapper, &mut frame_allocator)
        .expect("Heap initialization failed");
    println!("Allocator initialized");

    // ✅ Interrupts and LAPIC setup
   match kernel::init(&*boot_info.memory_regions, VirtAddr::new(boot_info.physical_memory_offset.into_option().unwrap())) {
    Ok(_) => {}
    Err(e) => {
        println!("Interrupt init failed: {:?}", e);
        hlt_loop();
    }
}


    println!("INTERRUPTS INITIATED\n");

    // ✅ Task executor
    let mut executor = Executor::new();
    executor.spawn(Task::new(example_task()));
    //executor.spawn(Task::new(keyboard::print_keypresses()));
    executor.run();
   

}

// 🛑 Panic handler
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    unsafe {
        framebuffer::WRITER.force_unlock();
    }
    println!("{}", info);
    hlt_loop();
}



async fn async_number() -> u32 {
  42
}

async fn example_task() {
  let number = async_number().await;
  println!("async number: {}", number);
}
 
