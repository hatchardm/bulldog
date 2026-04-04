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

    info!(
        "GOP: initialized, resolution {}x{}",
        ctx.fb.width, ctx.fb.height
    );

    ctx.fb.clear();
    ctx.fb.fill_color(color::Color::BLUE);

    ctx.fb.draw_rect(50, 50, 200, 100, color::Color::RED);

    ctx.fb.draw_line(
        0,
        0,
        ctx.fb.width as isize - 1,
        ctx.fb.height as isize - 1,
        color::Color::WHITE,
    );

    ctx.fb.draw_line(
        0,
        200,
        ctx.fb.width as isize - 1,
        200,
        color::Color::GREEN,
    );

    ctx.fb.draw_line(
        300,
        0,
        300,
        ctx.fb.height as isize - 1,
        color::Color::RED,
    );

    info!("GOP: framebuffer filled blue");
    info!("Reached point B (after GOP)");

    boot::stall(Duration::from_secs(5));

    Status::SUCCESS
}



