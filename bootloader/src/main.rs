#![no_std]
#![no_main]

use core::time::Duration;
use log::info;
use uefi::boot;
use uefi::prelude::*;

mod gop;
mod framebuffer;
mod color;


#[entry]
fn main() -> Status {
    uefi::helpers::init().unwrap();

    info!("Bulldog UEFI bootloader starting...");
    info!("Reached point A");

    let mut ctx = match gop::init() {
        Ok(c) => c,
        Err(status) => return status,
    };

    info!("GOP: initialized, resolution {}x{}", ctx.fb.width, ctx.fb.height);

    ctx.fb.clear(); // clears to black
    ctx.fb.fill_color(color::Color::BLUE); // then fill blue

    ctx.fb.draw_rect(50, 50, 200, 100, color::Color::RED);

    // Diagonal line
    ctx.fb.draw_line(0, 0, ctx.fb.width as isize - 1, ctx.fb.height as isize - 1, color::Color::WHITE);

    // Horizontal line
    ctx.fb.draw_line(0, 200, ctx.fb.width as isize - 1, 200, color::Color::GREEN);

    // Vertical line
    ctx.fb.draw_line(300, 0, 300, ctx.fb.height as isize - 1, color::Color::RED);



    info!("GOP: framebuffer filled blue");
    info!("Reached point B (after GOP)");

    boot::stall(Duration::from_secs(5));

    Status::SUCCESS
}
