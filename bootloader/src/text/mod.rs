use crate::framebuffer::Framebuffer;
use crate::color::Color;

// Embed the font directly into the binary
// Adjust the path if font8x16.bin lives elsewhere
static FONT8X16: &[u8] = include_bytes!("font8x16.bin");

pub fn load_font() -> &'static [u8] {
    FONT8X16
}

pub fn put_char(
    fb: &mut Framebuffer,
    font: &[u8],
    x: usize,
    y: usize,
    ch: u8,
    color: Color,
) {
    let glyph_offset = (ch as usize) * 16;

    for row in 0..16 {
        let row_bits = font[glyph_offset + row];

        for col in 0..8 {
            if (row_bits << col) & 0x80 != 0 {
                fb.put_pixel(x + col, y + row, color);
            }
        }
    }
}

pub fn write_str(
    fb: &mut Framebuffer,
    font: &[u8],
    mut x: usize,
    mut y: usize,
    text: &str,
    color: Color,
) {
    for b in text.bytes() {
        match b {
            b'\n' => {
                x = 0;
                y += 16;
            }
            _ => {
                put_char(fb, font, x, y, b, color);
                x += 8;
            }
        }
    }
}



