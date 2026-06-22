// kernel/src/font8x16.rs
#![allow(dead_code)]

pub const FONT_WIDTH: usize = 8;
pub const FONT_HEIGHT: usize = 16;

// Very small demo font: only 'H', 'E', 'L', 'O' for now.
// Each u8 is one row, bits 7..0 = pixels left→right.
pub static FONT8X16: [[u8; FONT_HEIGHT]; 128] = {
    let mut table = [[0u8; FONT_HEIGHT]; 128];

    // 'H' (0x48)
    table[b'H' as usize] = [
        0b10000001,
        0b10000001,
        0b10000001,
        0b11111111,
        0b10000001,
        0b10000001,
        0b10000001,
        0b10000001,
        0,0,0,0,0,0,0,0,
    ];

    // 'E' (0x45)
    table[b'E' as usize] = [
        0b11111111,
        0b10000000,
        0b10000000,
        0b11111110,
        0b10000000,
        0b10000000,
        0b11111111,
        0b00000000,
        0,0,0,0,0,0,0,0,
    ];

    // 'L' (0x4C)
    table[b'L' as usize] = [
        0b10000000,
        0b10000000,
        0b10000000,
        0b10000000,
        0b10000000,
        0b10000000,
        0b11111111,
        0b00000000,
        0,0,0,0,0,0,0,0,
    ];

    // 'O' (0x4F)
    table[b'O' as usize] = [
        0b01111110,
        0b10000001,
        0b10000001,
        0b10000001,
        0b10000001,
        0b10000001,
        0b01111110,
        0b00000000,
        0,0,0,0,0,0,0,0,
    ];

    table
};
