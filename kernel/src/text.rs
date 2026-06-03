use crate::framebuffer::KernelFramebuffer;
use crate::font::FONT8X8; // adjust path to where your font lives

pub fn draw_block_test(fb: &mut KernelFramebuffer, x: usize, y: usize, fg: u32) {
    for row in 0..8 {
        for col in 0..8 {
            fb.draw_pixel(x + col, y + row, fg);
        }
    }
}

pub fn draw_char_8x8(
    fb: &mut KernelFramebuffer,
    x: usize,
    y: usize,
    _ch: char,
    fg: u32,
    _bg: u32,
) {
    // Big obvious block: 100×100
    for row in 0..100 {
        for col in 0..100 {
            fb.draw_pixel(x + col, y + row, fg);
        }
    }
}





/* 
pub fn draw_char_8x8(
    fb: &mut KernelFramebuffer,
    x: usize,
    y: usize,
    ch: char,
    fg: u32,
    bg: u32,
) {
    let code = ch as usize;

    // FONT8X8 covers ASCII 0x20..0x7F
    if code < 0x20 || code > 0x7F {
        return;
    }

    let glyph = FONT8X8[code - 0x20];

    for row in 0..8 {
        let bits = glyph[row];

        for col in 0..8 {
            let on = (bits >> (7 - col)) & 1 != 0;
            let color = if on { fg } else { bg };
            fb.draw_pixel(x + col, y + row, color);
        }
    }
}
*/