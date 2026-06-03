//! Bulldog kernel entry point (`main.rs`).


#![no_std]
#![no_main]
#![allow(warnings)]

//extern crate alloc;


use core::panic::PanicInfo;
//use x86_64::instructions::port::Port;
//use alloc::string::ToString;

use kernel_lib::framebuffer::KernelFramebuffer;
    //writer,
   // color::*,
//};

use boot_proto::BootInfo as ProtoBootInfo;


//use kernel::time;
use kernel_lib::serial::{serial_print, serial_print_u64, serial_print_hex_u64};
use kernel_lib::text::{draw_block_test, draw_char_8x8};
use kernel_lib::console::Console;
use x86_64::VirtAddr;



//use kernel::logger::set_framebuffer_ready;
//use core::fmt::Write;
//use log::{info, error};
//use log::LevelFilter;
//use x86_64::VirtAddr;
//use kernel::writer::TextWriter;


// Bulldog kernel entry point (`main.rs`).

//#![no_std]
//#![no_main]
//#![allow(warnings)]

//extern crate alloc;

//use boot_proto::BootInfo;
//use boot_proto::Framebuffer;
//use core::panic::PanicInfo;
//use kernel::framebuffer::KernelFramebuffer;
//use kernel::color::*;



#[unsafe(no_mangle)]
pub extern "sysv64" fn bulldog_entry(boot_info: *mut ProtoBootInfo) -> ! {
    kernel_main(boot_info)
}

#[unsafe(no_mangle)]
pub extern "sysv64" fn kernel_main(boot_info: *mut ProtoBootInfo) -> ! {
    serial_print("KERNEL: entered kernel_main\n");

    let boot: &ProtoBootInfo = unsafe { &*boot_info };

    if boot.framebuffer_present == 0 {
        serial_print("KERNEL: no framebuffer present\n");
        loop { unsafe { core::arch::asm!("hlt"); } }
    }

    serial_print("KERNEL: framebuffer present\n");


    let phys_mem_offset = VirtAddr::new(boot.physical_memory_offset);
    let mut fb = KernelFramebuffer::from_bootinfo(boot, phys_mem_offset);

    //let mut fb = KernelFramebuffer::from_bootinfo(boot);

    serial_print("KERNEL: fb w=");
    serial_print_u64(fb.width as u64);
    serial_print(" h=");
    serial_print_u64(fb.height as u64);
    serial_print("\n");

    fb.clear_fast(0x00000000);
    serial_print("KERNEL: screen cleared\n");

    // --- console test ---

serial_print("KERNEL: before Console::new\n");
let mut console = Console::new(&mut fb);
serial_print("KERNEL: after Console::new\n");

serial_print("KERNEL: before console.write_str\n");
console.write_str("Hello");
serial_print("KERNEL: after console.write_str\n");

loop {
    unsafe { core::arch::asm!("hlt"); }
}




/* 
draw_char_8x8(&mut fb, 0, 0, 'X', 0x00FFFFFF, 0x00000000);
serial_print("KERNEL: after direct draw_char_8x8\n");

loop {
    unsafe { core::arch::asm!("hlt"); }
}
*/

    /*
    let mut console = Console::new(&mut fb);
    console.write_str("Hello");

    // direct test
    draw_char_8x8(&mut fb, 0, 0, 'X', 0x00FFFFFF, 0x00000000);
    serial_print("KERNEL: after direct draw_char_8x8\n");

    loop {
        unsafe { core::arch::asm!("hlt"); }
    }
    */
}




 




/* 
    let ptr = (HHDM_BASE + fb.addr as usize) as *mut u32;
    let stride = fb.stride;
    let height = fb.height;

    unsafe {
        if stride * height > 0 {
            *ptr = 0xFFFF0000;                 // red
            *ptr.add(stride + 1) = 0xFF00FF00; // green
        }
    }

    loop {
        unsafe { core::arch::asm!("hlt"); }
    }  
}  */



/* 
    serial_print("Before let mut fb = KernelFramebuffer::from_bulldog(framebuffer);\n");
    let mut fb = KernelFramebuffer::from_bulldog(framebuffer);
    serial_print("After let mut fb = KernelFramebuffer::from_bulldog(framebuffer);\n");
 
    serial_print("Before fb.clear_fast(BLACK);\n");
    fb.clear_fast(BLACK);
    serial_print("After fb.clear_fast(BLACK);\n");

    serial_print("Before let mut writer = TextWriter {... Block\n");
    let mut writer = TextWriter {
        fg: (255,255,255),
        bg: (0,0,0),
        x: 0,
        y: 0,
        width: 0,
        height: 0,
        stride: 0,
        fb: &mut [],
    };

    serial_print("After let mut writer = TextWriter {... Block\n");

    serial_print("Before writer::framebuffer_init(&mut fb, &mut writer);\n");
    writer::framebuffer_init(&mut fb, &mut writer);
    serial_print("After writer::framebuffer_init(&mut fb, &mut writer);\n");

    serial_print("Before writer.write_str_inner);\n");
      writer.write_str_inner("Bulldog Kernel Booting...\n");
    serial_print("After writer.write_str_inner)\n");

    loop {
        unsafe { core::arch::asm!("hlt"); }
    }  */









/* 
#[unsafe(no_mangle)]
pub extern "sysv64" fn kernel_main(boot_info: *mut BootInfo) -> ! {
       serial_print("KERNEL: entered kernel_main\n");




    // 🎨 Framebuffer setup
      serial_print("KERNEL: before framebuffer_init\n");
    let boot = unsafe { &mut *boot_info };

let framebuffer = boot
    .framebuffer
    .as_mut()
    .expect("BootInfo.framebuffer must be present");


    let mut fb = KernelFramebuffer::from_bulldog(framebuffer);
    serial_print("KERNEL: after framebuffer_init\n"); 
 

    fb.clear_fast(BLACK);

    serial_print("KERNEL: after clear_fast\n"); 


    
    // ✍️ Initialize WRITER
   //writer::framebuffer_init(&mut fb);
   serial_print("KERNEL: after writer::framebuffer_init\n");



let mut kfb = KernelFramebuffer::from_bulldog(framebuffer);

serial_print("KERNEL: after framebuffer_init\n");
clear_fast(&mut kfb);
serial_print("KERNEL: after clear_fast\n");

// comment out writer::framebuffer_init for this test
// writer::framebuffer_init(&mut kfb);

serial_print("KERNEL: before local writer\n");

let stride_pixels = kfb.pitch / 4;
let len = stride_pixels * kfb.height;

let fb_slice: &'static mut [u32] = unsafe {
    core::slice::from_raw_parts_mut(kfb.ptr as *mut u32, len)
};

let mut w = TextWriter {
    fg_color: (255, 255, 255),
    bg_color: (0, 0, 0),
    cursor_x: 0,
    cursor_y: 0,
    width: kfb.width,
    height: kfb.height,
    line_height: 16,
    stride_pixels,
    framebuffer: fb_slice,
    enable_scroll: true,
};

serial_print("KERNEL: before local banner\n");
w.write_str_inner("Bulldog Kernel Booting...\n");
serial_print("KERNEL: after local banner\n");


loop {
    unsafe { core::arch::asm!("hlt"); }
}



   //set_framebuffer_ready(true);
 
   //serial_print("KERNEL: after set_framebuffer_ready\n");

   
    // 🐾 Boot banner
  serial_print("KERNEL: before banner\n");

unsafe {
    if let Some(w) = (*WRITER.inner.get()).as_mut() {
        w.enable_scroll = true;
        w.set_color((255, 255, 255), (0, 0, 0));
        w.write_str_inner("Bulldog Kernel Booting...\n");
    }
}

serial_print("KERNEL: after banner\n");



   loop {
        unsafe { core::arch::asm!("hlt"); }
    }

//    if let Some(w) = unsafe { (*WRITER.inner.get()).as_mut() }
// {
//        #[cfg(feature = "syscall")]
//        let _ = writeln!(w, "[feature] syscall ENABLED");
//        #[cfg(not(feature = "syscall"))]
//        let _ = writeln!(w, "[feature] syscall DISABLED");

//        #[cfg(feature = "syscall_tests")]
//        let _ = writeln!(w, "[feature] syscall_tests ENABLED");
//        #[cfg(not(feature = "syscall_tests"))]
//        let _ = writeln!(w, "[feature] syscall_tests DISABLED");
//    }

    // 🪵 Logging
//    logger_init(LevelFilter::Info);
//    info!("Exited logger_init");
//    info!("Framebuffer format: {:?}, size: {}x{}", fb.pixel_format, fb.width, fb.height);

    // 🔠 Glyph diagnostics
 //   if let Some(glyph) = get_glyph('A') {
 //       info!("Glyph 'A' width={} height={}", glyph.width(), glyph.height());
 //   }

    // ✅ Prepare memory inputs for kernel_init
//    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);

    // NOTE: later we’ll switch kernel_init to use boot_proto::MemoryRegion
//    let memory_regions = boot_info.memory_regions;

//    match kernel_init(memory_regions, phys_mem_offset) {
//        Ok(_) => info!("kernel_init completed successfully"),
//        Err(e) => error!("kernel_init failed: {:?}", e),
//    }

//    info!("Returned to main");
//    hlt_loop();
}

*/






 // Panic handler.
 // Prints panic info over serial port, then halts in `hlt_loop`.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    
    unsafe {
        serial_print("KERNEL PANIC");

        if let Some(location) = info.location() {
            serial_print(" at ");
            serial_print(location.file());
            serial_print(":");
            serial_print_u64(location.line() as u64);
        }

        serial_print("\n");
    }

    loop {
        unsafe { core::arch::asm!("hlt"); }
    }
}









