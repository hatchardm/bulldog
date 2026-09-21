use crate::framebuffer::KernelFramebuffer;
use crate::color::Color;
use crate::font8x16::font8x16;

pub fn draw_char_8x16(
    fb: &mut KernelFramebuffer,
    x: usize,
    y: usize,
    ch: u8,
    fg: Color,
    bg: Color,
) {
    let font = font8x16(); // 4096-byte glyph table

    let glyph_offset = (ch as usize) * 16;

    for row in 0..16 {
        let row_bits = font[glyph_offset + row];

        for col in 0..8 {
            let bit = (row_bits >> (7 - col)) & 1;

            let px = x + col;
            let py = y + row;

            if px >= fb.width || py >= fb.height {
                continue;
            }

            let color = if bit != 0 { fg } else { bg };
            fb.put_pixel_color(px, py, color);
        }
    }
}


