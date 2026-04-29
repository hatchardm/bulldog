//! Bulldog kernel entry point (`main.rs`).


//#![no_std]
//#![no_main]
//#![allow(warnings)]

//extern crate alloc;

//use boot_proto::BootInfo as BulldogBootInfo;

//use core::panic::PanicInfo;
//use x86_64::instructions::port::Port;
//use alloc::string::ToString;

//use kernel::{
//    framebuffer::KernelFramebuffer,
//    writer::{self, WRITER},
//    font::get_glyph,
//    color::*,
//    hlt_loop,
//    logger::logger_init,
//    kernel_init,
//};
//use kernel::time;
//use kernel::serial::serial_print;
//use kernel::logger::set_framebuffer_ready;
//use core::fmt::Write;
//use log::{info, error};
//use log::LevelFilter;
//use x86_64::VirtAddr;

//! Bulldog kernel entry point (`main.rs`).

#![no_std]
#![no_main]
#![allow(warnings)]

use boot_proto::BootInfo;
use boot_proto::Framebuffer;
use core::panic::PanicInfo;

struct KernelFramebuffer {
    addr: u64,
    width: usize,
    height: usize,
    stride: usize,
}

impl KernelFramebuffer {
    fn from_boot(fb: &Framebuffer) -> Self {
        Self {
            addr: fb.addr as u64,
            width: fb.width,
            height: fb.height,
            stride: fb.stride,
        }
    }

    fn clear_fast(&mut self, color: u32) {
    let bytes_per_pixel = 4usize;
    let base = self.addr;

    let mut y = 0usize;
    while y < self.height {
        let mut x = 0usize;
        while x < self.width {
            let offset_pixels = y.wrapping_mul(self.stride).wrapping_add(x);
            let offset_bytes = (offset_pixels.wrapping_mul(bytes_per_pixel)) as u64;
            let addr = base.wrapping_add(offset_bytes);

            unsafe {
                core::arch::asm!(
                    "mov rax, {addr}",
                    "mov edx, {color:e}",
                    "mov dword ptr [rax], edx",
                    addr = in(reg) addr,
                    color = in(reg) color,
                    options(nostack, preserves_flags),
                );
            }

            x = x.wrapping_add(1);
        }
        y = y.wrapping_add(1);
    }
}

fn put_pixel(&mut self, x: usize, y: usize, color: u32) {
        if x >= self.width || y >= self.height {
            return;
        }

        let bytes_per_pixel = 4usize;
        let offset_pixels = y.wrapping_mul(self.stride).wrapping_add(x);
        let offset_bytes = (offset_pixels.wrapping_mul(bytes_per_pixel)) as u64;
        let addr = self.addr.wrapping_add(offset_bytes);

        unsafe {
            core::arch::asm!(
                "mov rax, {addr}",
                "mov edx, {color:e}",
                "mov dword ptr [rax], edx",
                addr = in(reg) addr,
                color = in(reg) color,
                options(nostack, preserves_flags),
            );
        }
    }


}


#[unsafe(no_mangle)]
pub extern "sysv64" fn bulldog_entry(boot_info: *mut BootInfo) -> ! {
    // this is the symbol the linker uses as the entry point
    kernel_main(boot_info)
}

#[unsafe(no_mangle)]
pub extern "sysv64" fn kernel_main(boot_info: *mut BootInfo) -> ! {
    let boot = unsafe { &mut *boot_info };

    if let Some(ref mut fb) = boot.framebuffer {
        let mut kfb = KernelFramebuffer::from_boot(fb);

        // clear to black
        kfb.clear_fast(0x00000000);

        // draw a white rectangle in the top-left
        let rect_w = 100;
        let rect_h = 50;
        let color = 0x00FFFFFFu32; // white (BGR/BGRA)

        let mut y = 0usize;
        while y < rect_h && y < kfb.height {
            let mut x = 0usize;
            while x < rect_w && x < kfb.width {
                kfb.put_pixel(x, y, color);
                x += 1;
            }
            y += 1;
        }
    }

    loop {
        unsafe { core::arch::asm!("hlt"); }
    }
}



#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {
        unsafe { core::arch::asm!("hlt"); }
    }
}










//fn kernel_main(boot_info: &'static mut BulldogBootInfo) -> ! {
//   unsafe {
//        let mut port = Port::<u8>::new(0x3F8);
//        for &b in b"KERNEL: entered kernel_main\n" {
//            port.write(b);
//        }
//    }

//    loop {}


    // 🎨 Framebuffer setup
//     unsafe { serial_print("KERNEL: before framebuffer_init\n"); }
//    let framebuffer = boot_info
//        .framebuffer
//        .as_mut()
//        .expect("BootInfo.framebuffer must be present");

//    let mut fb = KernelFramebuffer::from_bulldog(framebuffer);
//unsafe { serial_print("KERNEL: after framebuffer_init\n"); }


//    fb.clear_fast(BLACK);

//unsafe { serial_print("KERNEL: after clear_fast\n"); }
    

    // ✍️ Initialize WRITER
//    writer::framebuffer_init(&mut fb);

//      unsafe { serial_print("KERNEL: after writer::framebuffer_init\n"); }

//    set_framebuffer_ready(true);

//   unsafe { serial_print("KERNEL: after set_framebuffer_ready\n"); }

    // 🐾 Boot banner
//    if let Some(w) = WRITER.lock().as_mut() {
//        w.enable_scroll = true;
//        w.set_color((255, 255, 255), (0, 0, 0));
//        let _ = writeln!(w, "🐾 Bulldog Kernel Booting...");
//    }

//  unsafe { serial_print("KERNEL: after boot banner\n"); }

//    if let Some(w) = WRITER.lock().as_mut() {
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
//}


 // Panic handler.
 // Prints panic info over serial port, then halts in `hlt_loop`.
//#[panic_handler]
//fn panic(info: &PanicInfo) -> ! {
//    unsafe {
//        serial_print("KERNEL PANIC: ");
//        if let Some(location) = info.location() {
//            serial_print(" at ");
//            serial_print(location.file());
//            serial_print(":");
//            serial_print(&location.line().to_string());
//        }
//        serial_print("\n");
//    }
//    hlt_loop();
//}








