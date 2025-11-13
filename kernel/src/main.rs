#![no_std]
#![no_main]
#![allow(warnings)]

#[macro_use]
extern crate kernel;

extern crate alloc;

use bootloader_api::{
    config::{BootloaderConfig, Mapping},
    entry_point,
    info::BootInfo,
};
use core::panic::PanicInfo;
use log::LevelFilter;
use x86_64::instructions::port::Port;
use alloc::string::ToString;

use kernel::{
    framebuffer::{self, KernelFramebuffer},
    writer::{self, WRITER, TextWriter},
    font::get_glyph,
    color::*,
    hlt_loop,
    logger::KernelLogger,
};
 use kernel::logger::logger_init;
 use core::fmt::Write;

static LOGGER: KernelLogger = KernelLogger;

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
    writer::init(&mut fb);

if let Some(writer) = WRITER.lock().as_mut() {
    writer.enable_scroll = false; // disable scrolling
}


    

if let Some(ref mut writer) = WRITER.lock().as_mut() {
    writer.enable_scroll = true; // explicitly enable scrolling
    writer.set_color((255, 255, 255), (0, 0, 0)); // white on black
    writer.write_str("🐾 Bulldog Kernel Booting...\n").unwrap();
} // lock released here

    
  //  writer::boot_log(&format!(
   // "Framebuffer format: {:?}, size: {}x{}",
   // fb.pixel_format, fb.width, fb.height
//));
    
    // 🪵 Logging
    logger_init();

    if let Some(glyph) = get_glyph('A') {
    log::info!("Glyph 'A' width={} height={}", glyph.width(), glyph.height());
    log::info!("Raster rows: {}", glyph.raster().len());
    for (i, row) in glyph.raster().iter().enumerate() {
        log::info!("Row {} has {} bytes", i, row.len());
    }
}

    log::debug!("Debugging enabled");
    log::info!("System boot complete");
    log::warn!("Low memory warning");
    log::error!("Page fault at 0xdeadbeef");
    log::trace!("Trace message for detailed debugging");

    // 🔠 Glyph info
    if let Some(glyph) = get_glyph('A') {
        log::info!("Glyph size: {}x{}", glyph.width(), glyph.height());
        log::info!("Raster rows: {}", glyph.raster().len());
    }


    // ✅ Test output
    println!("Hello\nWorld");

   


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



