#![no_std]
#![no_main]

use core::time::Duration;
use log::info;
use uefi::boot;
use uefi::prelude::*;

mod gop;

#[entry]
fn main() -> Status {
    uefi::helpers::init().unwrap();

    info!("Bulldog UEFI bootloader starting...");
    info!("Reached point A");

    let mut ctx = match gop::init() {
        Ok(c) => c,
        Err(status) => return status,
    };

    info!("GOP: initialized, resolution {}x{}", ctx.width, ctx.height);

    // New: use helper
    ctx.fill_color(0x00, 0x00, 0xFF); // blue

    info!("GOP: framebuffer filled blue");
    info!("Reached point B (after GOP)");

    boot::stall(Duration::from_secs(5));

    Status::SUCCESS
}

