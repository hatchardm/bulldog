use crate::framebuffer::KernelFramebuffer;
use crate::text::draw_char_8x16;

pub struct TextWriter<'a> {
    pub fb: &'a mut KernelFramebuffer,
    pub fg: u32,
    pub bg: u32,
    pub x: usize,
    pub y: usize,
    pub char_w: usize,
    pub char_h: usize,
}

impl<'a> TextWriter<'a> {
    pub fn write_str(&mut self, s: &str) {
        for b in s.bytes() {
            self.write_char(b);
        }
    }

    pub fn write_char(&mut self, ch: u8) {
        draw_char_8x16(self.fb, ch, self.x, self.y, self.fg, self.bg);
        self.x += self.char_w;

        // crude wrap
        if self.x + self.char_w >= self.fb.width {
            self.x = 0;
            self.y += self.char_h;
        }
    }
}


