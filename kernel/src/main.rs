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
use x86_64::VirtAddr;

use kernel::{
    framebuffer,
    hlt_loop,
    init,
    interrupts::LAPIC_HITS_RAW,
    task::{executor::Executor, keyboard, Task},
    time,
};

use core::sync::atomic::{AtomicUsize, Ordering};
use alloc::string::ToString;
use x86_64::instructions::port::Port;
use kernel::framebuffer::KernelFramebuffer;
use kernel::font;
use kernel::color::*;
use kernel::logger::KernelLogger;
use log::LevelFilter;
use kernel::writer::WRITER;

static LOGGER: KernelLogger = KernelLogger;

// 🛠 Bootloader configuration
const CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();
    config.kernel_stack_size = 100 * 1024;
    config.mappings.physical_memory = Some(Mapping::Dynamic);
    config.mappings.framebuffer = Mapping::Dynamic;
    config
};

entry_point!(kernel_main, config = &CONFIG);

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    let framebuffer = boot_info.framebuffer.as_mut().unwrap();
    let mut fb = KernelFramebuffer::from_bootloader(framebuffer);
    fb.clear_fast(BLACK);
    kernel::writer::init(&mut fb);
    logger_init(); // Now logging will work

    log::info!("Logger is live — Bulldog booting...");
    println!("Framebuffer is live at {}x{}\n", fb.width, fb.height);

    fb.draw_rect(0, 0, 20, 20, WHITE);

if let Some(glyph) = crate::font::get_glyph('A') {
    println!("Glyph size: {}x{}", glyph.width(), glyph.height());
    serial_print("Reached after println\n");

    let raster = glyph.raster();

    // Check WRITER state before locking
    if let Some(_) = WRITER.try_lock().as_deref() {
        serial_print("WRITER is Some and lockable\n");
    } else {
        serial_print("WRITER is None or lock poisoned\n");
    }

    // Lock WRITER and extract data
    let (base_x, base_y, tw_exists) = {
        let mut writer = WRITER.lock();

        if let Some(ref mut tw) = *writer {
            let x = tw.x;
            let y = tw.y;
            tw.x += glyph.width(); // simulate advance
            (x, y, true)
        } else {
            (0, 0, false)
        }
    }; // 🔓 lock released here

    // Now it's safe to log
    log::info!("WRITER lock succeeded");

    if tw_exists {
        log::info!("TextWriter is Some");
        log::info!("Cursor position: x={}, y={}", base_x, base_y);
    } else {
        log::warn!("TextWriter is None");
    }

    log::info!("Manual glyph draw block completed");
}



println!("Hello\nWorld");




    let fb_info = framebuffer.info();
    let buffer = framebuffer.buffer_mut();
    let pixel_ptr = buffer.as_mut_ptr() as *mut u32;

    for y in 0..20 {
        for x in 0..20 {
            let idx = y * fb_info.stride + x;
            unsafe {
                pixel_ptr.add(idx).write_volatile(0x00FFFFFF);
            }
        }
    }

    unsafe {
        let mut port = Port::new(0x3F8);
        port.write(b'K');
    }

    loop {
        unsafe { core::arch::asm!("hlt"); }
    }
}

pub fn logger_init() {
    log::set_logger(&LOGGER).unwrap();
    log::set_max_level(LevelFilter::Info);
    log::info!("Logger initialized");
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

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial_print("KERNEL PANIC: ");

    if let Some(location) = info.location() {
        serial_print(" at ");
        serial_print(location.file());
        serial_print(":");
        serial_print(&location.line().to_string());
    }

    serial_print("\n");
    hlt_loop();
}

async fn async_number() -> u32 {
    42
}

async fn example_task() {
    let number = async_number().await;
    // println!("async number: {}", number);
}

