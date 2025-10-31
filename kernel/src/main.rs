#![no_std]
#![no_main]
#![allow(warnings)]
extern crate alloc;

use bootloader_api::{
    config::{BootloaderConfig, Mapping},
    entry_point,
    info::BootInfo,
};
use core::panic::PanicInfo;
use x86_64::VirtAddr;

use kernel::{
    framebuffer,
    hlt_loop,
    task::{executor::Executor, keyboard, Task},
    println,
};

use kernel::init;
use kernel::time;
use core::sync::atomic::{AtomicUsize, Ordering};
use kernel::interrupts::LAPIC_HITS_RAW;


// 🛠 Bootloader configuration

const CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();
    config.kernel_stack_size = 100 * 1024; // 100 KiB
    config.mappings.physical_memory = Some(Mapping::Dynamic);
    config
};


entry_point!(kernel_main, config = &CONFIG);

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    // Framebuffer first—enables println!()
    let framebuffer = boot_info.framebuffer.as_mut().unwrap();

// Extract info before init to avoid borrow conflict
let fb_info = framebuffer.info();
let fb_ptr = framebuffer.buffer().as_ptr() as usize;

framebuffer::init(framebuffer); // Must come before println! so output works
const KERNEL_VERSION: &str = "v0.1.0";
println!("🐾 Bulldog Kernel {} — Ready to pounce.", KERNEL_VERSION);


println!("Framebuffer initialized"); // after framebuffer::init
println!("Framebuffer physical address: {:#x}", fb_ptr);
println!("Framebuffer range: {:#x} - {:#x}", fb_ptr, fb_ptr + fb_info.byte_len);

println!("Extracting memory info early to avoid borrow conflicts");
    // Extract memory info early to avoid borrow conflicts
    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset.into_option().unwrap());
println!("Extracted memory info early to avoid borrow conflicts");

//Debug code
//println!(
    //"Memory regions ptr: {:p}",
   // &boot_info.memory_regions as *const _
//);

println!("Calling init");
let result = init(&*boot_info.memory_regions, phys_mem_offset);
println!("Init result: {:?}", result);

println!("Exited init");

for _ in 0..1_000_000 {
    core::hint::spin_loop();
}

println!("LAPIC_HITS_RAW addr: {:p}", unsafe { &raw const LAPIC_HITS_RAW });







 


    // ✅ Task executor
    let mut executor = Executor::new();
    executor.spawn(Task::new(example_task()));
    //executor.spawn(Task::new(keyboard::print_keypresses()));
    println!("Bulldog kernel boot complete. Entering task executor.");
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
 
