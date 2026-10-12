use crate::framebuffer::Framebuffer;
use crate::color::Color;
use crate::text::put_char;
use core::slice;

pub struct Console<'a> {
    fb: &'a mut Framebuffer,
    font: &'static [u8],

    cursor_x: usize,
    cursor_y: usize,
    cols: usize,
    rows: usize,
    char_w: usize,
    char_h: usize,
    fg: Color,
}

impl<'a> Console<'a> {
    pub fn new(fb: &'a mut Framebuffer, font: &'static [u8]) -> Self {
        let char_w = 8;
        let char_h = 16;

        let cols = fb.width / char_w;
        let rows = fb.height / char_h;

        let mut console = Console {
            fb,
            font,
            cursor_x: 0,
            cursor_y: 0,
            cols,
            rows,
            char_w,
            char_h,
            fg: Color::WHITE,
        };

        console.clear();
        console
    }

    pub fn set_color(&mut self, color: Color) {
        self.fg = color;
    }

    pub fn clear(&mut self) {
        self.fb.clear();
        self.cursor_x = 0;
        self.cursor_y = 0;
    }

    fn newline(&mut self) {
        self.cursor_x = 0;
        self.cursor_y += 1;

        if self.cursor_y >= self.rows {
            self.scroll();
            self.cursor_y = self.rows - 1;
        }
    }

    fn scroll(&mut self) {
        let row_bytes = self.char_h * self.fb.stride * 4;
        let screen_bytes = self.fb.height * self.fb.stride * 4;

        let src_start = row_bytes;
        let dst_start = 0;
        let len = screen_bytes - row_bytes;

        unsafe {
            let data = slice::from_raw_parts_mut(self.fb.ptr, self.fb.size);

            // Scroll framebuffer contents upward
            data.copy_within(src_start..src_start + len, dst_start);

            // Clear the last row
            let last_row_start = screen_bytes - row_bytes;
            for b in &mut data[last_row_start..screen_bytes] {
                *b = 0;
            }
        }
    }

    fn put_char_at_cursor(&mut self, ch: char) {
        if ch == '\n' {
            self.newline();
            return;
        }

        if self.cursor_x >= self.cols {
            self.newline();
        }

        let x = self.cursor_x * self.char_w;
        let y = self.cursor_y * self.char_h;

    

        // Draw glyph
        put_char(self.fb, self.font, x, y, ch as u8, self.fg);

        self.cursor_x += 1;
        if self.cursor_x >= self.cols {
            self.newline();
        }
    }

    pub fn write_str(&mut self, s: &str) {
        for ch in s.chars() {
            self.put_char_at_cursor(ch);
        }
    }
}
