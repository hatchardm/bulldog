//! Bulldog kernel entry point (`main.rs`).
//!
//! - `#![no_std]`: no standard library, only `core`.
//! - `#![no_main]`: custom entry point via `bootloader_api::entry_point!`.
//! - Configures bootloader stack size and memory mappings.
//! - Initializes framebuffer, writer, logger, and kernel subsystems.
//! - Hands off to `kernel_init` for paging/APIC setup.
//! - Drops into `hlt_loop` as the idle routine.

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
use x86_64::instructions::port::Port;
use alloc::string::ToString;

use kernel::{
    framebuffer::KernelFramebuffer,
    writer::{self, WRITER},
    font::get_glyph,
    color::*,
    hlt_loop,
    logger::logger_init,
    kernel_init,
};
use kernel::time;
use kernel::serial::serial_print;
use kernel::logger::set_framebuffer_ready;
use core::fmt::Write;
use log::{info, error};
use log::LevelFilter;
use x86_64::VirtAddr;

/// Bootloader configuration.
/// - Kernel stack size: 100 KiB
/// - Physical memory mapping: dynamic
/// - Framebuffer mapping: dynamic
const CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();
    config.kernel_stack_size = 100 * 1024;
    config.mappings.physical_memory = Some(Mapping::Dynamic);
    config.mappings.framebuffer = Mapping::Dynamic;
    config
};

/// Kernel entry point invoked by the bootloader.
/// 
/// - Initializes framebuffer and writer.
/// - Prints boot banner.
/// - Sets up logging.
/// - Runs glyph diagnostics.
/// - Calls `kernel_init` for paging/APIC setup.
/// - Drops into `hlt_loop` idle routine.
entry_point!(kernel_main, config = &CONFIG);

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    // 🎨 Framebuffer setup
    let framebuffer = boot_info.framebuffer.as_mut().expect("BootInfo.framebuffer must be present");
    let mut fb = KernelFramebuffer::from_bootloader(framebuffer);
    fb.clear_fast(BLACK);

    // ✍️ Initialize WRITER
    writer::framebuffer_init(&mut fb);
    set_framebuffer_ready(true);


    // 🐾 Boot banner
    if let Some(w) = WRITER.lock().as_mut() {
        w.enable_scroll = true;
        w.set_color((255, 255, 255), (0, 0, 0));
        let _ = writeln!(w, "🐾 Bulldog Kernel Booting...");
    }

    if let Some(w) = WRITER.lock().as_mut() {
    #[cfg(feature = "syscall")]
    let _ = writeln!(w, "[feature] syscall ENABLED");
    #[cfg(not(feature = "syscall"))]
    let _ = writeln!(w, "[feature] syscall DISABLED");

    #[cfg(feature = "syscall_tests")]
    let _ = writeln!(w, "[feature] syscall_tests ENABLED");
    #[cfg(not(feature = "syscall_tests"))]
    let _ = writeln!(w, "[feature] syscall_tests DISABLED");
}

    // 🪵 Logging
    logger_init(LevelFilter::Info);
    info!("Exited logger_init");
    info!("Framebuffer format: {:?}, size: {}x{}", fb.pixel_format, fb.width, fb.height);
    

    // 🔠 Glyph diagnostics
    if let Some(glyph) = get_glyph('A') {
        info!("Glyph 'A' width={} height={}", glyph.width(), glyph.height());
    }

    // ✅ Prepare memory inputs for kernel_init
    let phys_mem_offset = VirtAddr::new(
        boot_info.physical_memory_offset.into_option()
            .expect("BootInfo must provide physical memory offset")
    );
    let memory_regions: &'static [bootloader_api::info::MemoryRegion] = &boot_info.memory_regions;

    match kernel_init(memory_regions, phys_mem_offset) {
        Ok(_) => info!("kernel_init completed successfully"),
        Err(e) => error!("kernel_init failed: {:?}", e),
    }


    

    info!("Returned to main");

    hlt_loop();
}

/// Panic handler.
/// Prints panic info over serial port, then halts in `hlt_loop`.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    unsafe {
        serial_print("KERNEL PANIC: ");
        if let Some(location) = info.location() {
            serial_print(" at ");
            serial_print(location.file());
            serial_print(":");
            serial_print(&location.line().to_string());
        }
        serial_print("\n");
    }
    hlt_loop();
}








