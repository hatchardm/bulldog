use crate::font::get_glyph;
use noto_sans_mono_bitmap::RasterizedChar;
use spin::Mutex;
use crate::framebuffer::KernelFramebuffer;
use core::fmt;

const LINE_SPACING: usize = 2;

pub static WRITER: Mutex<Option<TextWriter>> = Mutex::new(None);

pub struct TextWriter {
    pub x: usize,
    pub y: usize,
    pub framebuffer: &'static mut [u32],
    pub width: usize,
    pub height: usize,
    pub stride: usize, // bytes per row
    pub fg_color: u8,
    pub bg_color: u8,
}

impl TextWriter {
pub fn write_char(&mut self, c: char) {
    match c {
        '\n' => {
            self.x = 0;
            self.y += get_glyph(' ').map(|g| g.height()).unwrap_or(16) + LINE_SPACING;

            // ✅ Clamp vertical overflow
            if self.y + 16 > self.height {
                self.y = 0; // or scroll, or clear
            }
        }
        _ => {
            if let Some(glyph) = get_glyph(c) {
                if self.x + glyph.width() > self.width {
                    self.x = 0;
                    self.y += glyph.height() + LINE_SPACING;

                    // ✅ Clamp vertical overflow
                    if self.y + glyph.height() > self.height {
                        self.y = 0; // or scroll, or clear
                    }
                }

                self.draw_glyph(&glyph);
                self.x += glyph.width();
            }
        }
    }
}





    fn draw_glyph(&mut self, glyph: &RasterizedChar) {
        let glyph_w = glyph.width();
        let glyph_h = glyph.height();
        let raster = glyph.raster();

        for (dy, row) in raster.iter().enumerate() {
            for (dx, &alpha) in row.iter().enumerate() {
                let px = self.x + dx;
                let py = self.y + dy;
                if px < self.width && py < self.height {
                    let offset = py * self.stride + px;
                    self.framebuffer[offset] = self.blend_color(alpha);
                }
            }
        }
    }

    fn blend(&self, alpha: u8) -> u8 {
        ((self.fg_color as u16 * alpha as u16 + self.bg_color as u16 * (255 - alpha as u16)) / 255) as u8
    }



    fn blend_color(&self, alpha: u8) -> u32 {
        let fg = self.fg_color as u32;
        let bg = self.bg_color as u32;
        let blended = (fg * alpha as u32 + bg * (255 - alpha as u32)) / 255;

        // Replicate grayscale value across BGR channels
        (blended << 16) | (blended << 8) | blended
    }

}




pub fn init(fb: &mut KernelFramebuffer) {
    let writer = TextWriter {
        x: 0,
        y: 0,
        framebuffer: unsafe {
               core::slice::from_raw_parts_mut(fb.ptr as *mut u32, fb.pitch * fb.height)
   },
        width: fb.width,
        height: fb.height,
        stride: fb.pitch / 4, // convert bytes to pixels
        fg_color: 255,
        bg_color: 0,
    };
    *WRITER.lock() = Some(writer);
}

impl fmt::Write for TextWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.chars() {
            self.write_char(c);
        }
        Ok(())
    }
}
