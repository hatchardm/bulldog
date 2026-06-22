use crate::framebuffer::KernelFramebuffer;
use crate::font8x16::{FONT8X16, FONT_WIDTH, FONT_HEIGHT};

pub fn draw_char_8x16(
    fb: &mut KernelFramebuffer,
    ch: u8,
    x: usize,
    y: usize,
    fg: u32,
    bg: u32,
) {
    let glyph = &FONT8X16[ch as usize];

    for row in 0..FONT_HEIGHT {
        let bits = glyph[row];
        for col in 0..FONT_WIDTH {
            let mask = 1 << (7 - col);
            let color = if bits & mask != 0 { fg } else { bg };
            fb.put_pixel(x + col, y + row, color);
        }
    }
}
