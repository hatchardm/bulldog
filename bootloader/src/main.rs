#![no_std]
#![no_main]

extern crate alloc;

mod gop;
mod framebuffer;
mod color;
mod text;

use core::time::Duration;
use log::info;
use uefi::boot;
use uefi::prelude::*;
use text::{load_font, write_str};
use crate::color::Color;


#[entry]
fn main() -> Status {
    // Initialize UEFI logging + panic handler
    uefi::helpers::init().unwrap();

    info!("Bulldog UEFI bootloader starting…");

    //
    // Initialize GOP using your updated gop.rs
    //
    let mut ctx = match gop::init() {
        Ok(c) => c,
        Err(status) => return status,
    };

    info!(
        "GOP initialized: {}x{} stride {}",
        ctx.fb.width, ctx.fb.height, ctx.fb.stride
    );


    let font = load_font();

    //
    // Draw using your existing framebuffer API
    //
    ctx.fb.clear();                          // Clear screen (your API takes no args)
    ctx.fb.fill_color(color::Color::BLUE);   // Fill entire screen blue

    // Red rectangle
    ctx.fb.draw_rect(50, 50, 200, 100, color::Color::RED);

    // White diagonal line
    ctx.fb.draw_line(
        0,
        0,
        ctx.fb.width as isize - 1,
        ctx.fb.height as isize - 1,
        color::Color::WHITE,
    );

    // Green horizontal line
    ctx.fb.draw_line(
        0,
        200,
        ctx.fb.width as isize - 1,
        200,
        color::Color::GREEN,
    );

    // Red vertical line
    ctx.fb.draw_line(
        300,
        0,
        300,
        ctx.fb.height as isize - 1,
        color::Color::RED,
    );


    write_str(
    &mut ctx.fb,
    font,
    20,
    20,
    "Bulldog Bootloader\nText rendering online!",
    Color::WHITE,
);

    info!("Drawing complete. Stalling for 5 seconds…");

    boot::stall(Duration::from_secs(5));

    Status::SUCCESS
}




