use crate::framebuffer::KernelFramebuffer;
use crate::font8x16::FONT8X16;

pub unsafe fn draw_char_8x16(
    fb: &mut KernelFramebuffer,
    x: usize,
    y: usize,
    ch: u8,
    fg: u32,
    bg: u32,
) {
    let glyph = FONT8X16[ch as usize];

    let fb_ptr = fb.ptr;
    let stride = fb.pitch;

    for row in 0..16 {
        let row_bits = glyph[row];

        for col in 0..8 {
            let bit = (row_bits >> (7 - col)) & 1;
            let color = if bit != 0 { fg } else { bg };

            let px = x + col;
            let py = y + row;

            if px >= fb.width || py >= fb.height {
                continue;
            }

            let offset = (py * stride + px * 4) as isize;
            let p = fb_ptr.offset(offset);

            // BGRX
            *p.offset(0) = (color & 0x0000FF) as u8;
            *p.offset(1) = ((color >> 8) & 0xFF) as u8;
            *p.offset(2) = ((color >> 16) & 0xFF) as u8;
            *p.offset(3) = 0;
        }
    }
}

