use crate::text::draw_char_8x16;
use crate::framebuffer::KernelFramebuffer;
use crate::color::Color;
use spin::Mutex;

pub static WRITER: Mutex<Option<Writer>> = Mutex::new(None);

#[derive(Copy, Clone)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}


pub struct Writer {
    pub fb: *mut KernelFramebuffer,
    pub fg: Color,
    pub bg: Color,
    pub x: usize,
    pub y: usize,
    pub char_w: usize,   // 8
    pub char_h: usize,   // 16
}

impl Writer {
    pub fn new(fb: &mut KernelFramebuffer) -> Self {
        Self {
            fb,
            fg: Color::WHITE,
            bg: Color::BLACK,
            x: 0,
            y: 0,
            char_w: 8,
            char_h: 16,
        }
    }

    #[inline(always)]
    fn fb_mut(&mut self) -> &mut KernelFramebuffer {
        unsafe { &mut *self.fb }
    }

    pub fn write_str(&mut self, s: &str) {
        for ch in s.chars() {
            self.write_char(ch);
        }
    }

pub fn write_char(&mut self, ch: char) {
    // 1. Horizontal wrap BEFORE drawing
    if self.x + self.char_w >= self.fb_mut().width {
        self.x = 0;
        self.y += self.char_h;
    }

    // 2. Vertical scroll BEFORE drawing
    if self.y + self.char_h >= self.fb_mut().height {
        self.scroll_up();
    }

    // 3. Draw the character
    match ch {
        '\n' => {
            self.x = 0;
            self.y += self.char_h;
            return;
        }
        _ => {
            let ascii = ch as u8;

            let x = self.x;
            let y = self.y;
            let fg = self.fg;
            let bg = self.bg;

            unsafe {
                draw_char_8x16(
                    self.fb_mut(),
                    x,
                    y,
                    ascii,
                    fg,
                    bg,
                );
            }
        }
    }

    // 4. Advance cursor AFTER drawing
    self.x += self.char_w;
}


fn scroll_up(&mut self) {
    // Extract everything BEFORE calling fb_mut()
    let char_h = self.char_h;
    let bg = self.bg;

    let fb = self.fb_mut();

    let bytes_per_pixel = 4;
    let row_bytes = fb.pitch;
    let char_row_bytes = char_h * row_bytes;

    let total_bytes = fb.height * row_bytes;

    let src = fb.ptr;
    let dst = fb.ptr;

    unsafe {
        // Move framebuffer up by one character row (16px)
        core::ptr::copy(
            src.add(char_row_bytes),
            dst,
            total_bytes - char_row_bytes,
        );

        // Clear the bottom character row
        let clear_start = dst.add(total_bytes - char_row_bytes);
        let clear_color = fb.pack_color(bg.r, bg.g, bg.b);

        let mut p = clear_start as *mut u32;
        let pixels = char_row_bytes / bytes_per_pixel;

        for _ in 0..pixels {
            p.write_volatile(clear_color);
            p = p.add(1);
        }
    }

    // Cursor moves up by one character row
    self.y -= char_h;
}

 pub fn set_color(&mut self, fg: Color, bg: Color) {
        self.fg = fg;
        self.bg = bg;
    }

    pub fn set_log_level_color(&mut self, level: LogLevel) {
        match level {
            LogLevel::Error => self.set_color(Color::RED,   Color::BLACK),
            LogLevel::Warn  => self.set_color(Color::YELLOW, Color::BLACK),
            LogLevel::Info  => self.set_color(Color::WHITE, Color::BLACK),
            LogLevel::Debug => self.set_color(Color::CYAN,  Color::BLACK),
            LogLevel::Trace => self.set_color(Color::GRAY,  Color::BLACK),
        }
    }


}

impl core::fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.write_str(s);
        Ok(())
    }
}


unsafe impl Send for Writer {}
unsafe impl Sync for Writer {}

