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
use core::fmt::Write;
use log::{info, debug, warn, error, trace};
use log::LevelFilter;
use x86_64::VirtAddr;   // bring VirtAddr into scope

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
    let framebuffer = boot_info.framebuffer.as_mut().expect("BootInfo.framebuffer must be present");
    let mut fb = KernelFramebuffer::from_bootloader(framebuffer);
    fb.clear_fast(BLACK);

    // ✍️ Initialize WRITER
    writer::framebuffer_init(&mut fb);

    // 🐾 Boot banner
    if let Some(w) = WRITER.lock().as_mut() {
        w.enable_scroll = true;
        w.set_color((255, 255, 255), (0, 0, 0));
        let _ = writeln!(w, "🐾 Bulldog Kernel Booting...");
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

    info!("Returned to maim");



   hlt_loop();


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






