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
    framebuffer::{self, KernelFramebuffer},
    writer::{self, WRITER, TextWriter},
    font::get_glyph,
    color::*,
    hlt_loop,
    logger::logger_init,
};

use core::fmt::Write;
use log::{info, debug, warn, error, trace}; // log macros
use log::LevelFilter;

const CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();
    config.kernel_stack_size = 100 * 1024;
    config.mappings.physical_memory = Some(Mapping::Dynamic);
    config.mappings.framebuffer = Mapping::Dynamic;
    config
};

entry_point!(kernel_main, config = &CONFIG);

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    // 🎨 Framebuffer setup
    let framebuffer = boot_info.framebuffer.as_mut().unwrap();
    let mut fb = KernelFramebuffer::from_bootloader(framebuffer);
    fb.clear_fast(BLACK);

    // ✍️ Initialize WRITER
    writer::framebuffer_init(&mut fb);

    // 🐾 Boot banner: always white on black
    if let Some(w) = WRITER.lock().as_mut() {
        w.enable_scroll = true;
        w.set_color((255, 255, 255), (0, 0, 0)); // force white
        let _ = writeln!(w, "🐾 Bulldog Kernel Booting...");
    }

    // 🪵 Logging
    logger_init(LevelFilter::Warn); //Set filter level Info, Debug, Error, Warn and Trace
    info!("Framebuffer format: {:?}, size: {}x{}", fb.pixel_format, fb.width, fb.height);

    // 🔠 Glyph diagnostics
    if let Some(glyph) = get_glyph('A') {
        info!("Glyph 'A' width={} height={}", glyph.width(), glyph.height());
        info!("Raster rows: {}", glyph.raster().len());
        for (i, row) in glyph.raster().iter().enumerate() {
            info!("Row {} has {} bytes", i, row.len());
        }
    }

    debug!("Debugging enabled");
    info!("System boot complete");
    warn!("Low memory warning");
    error!("Page fault at 0xdeadbeef");
    trace!("Trace message for detailed debugging");

    // ✅ Test output
    info!("Hello");
    info!("World");

    loop {
        unsafe { core::arch::asm!("hlt"); }
    }
}

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

fn serial_write_byte(byte: u8) {
    unsafe {
        let mut port = Port::new(0x3F8);
        port.write(byte);
    }
}

fn serial_print(s: &str) {
    for byte in s.bytes() {
        serial_write_byte(byte);
    }
}






