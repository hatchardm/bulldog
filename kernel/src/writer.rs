use core::fmt::{self, Write, Arguments};
use spin::Mutex;
use crate::framebuffer::KernelFramebuffer;
use crate::font::get_glyph;
use noto_sans_mono_bitmap::RasterizedChar;
use core::slice;

/// Log levels for kernel logging
#[derive(Copy, Clone)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl LogLevel {
    pub fn prefix(&self) -> &'static str {
        match self {
            LogLevel::Trace => "[TRACE] ",
            LogLevel::Debug => "[DEBUG] ",
            LogLevel::Info  => "[INFO ] ",
            LogLevel::Warn  => "[WARN ] ",
            LogLevel::Error => "[ERROR] ",
        }
    }
}

pub struct TextWriter {
    pub fg_color: (u8, u8, u8),
    pub bg_color: (u8, u8, u8),
    pub cursor_x: usize,
    pub cursor_y: usize,
    pub width: usize,
    pub height: usize,
    pub line_height: usize,
    pub framebuffer: &'static mut [u32],
    pub enable_scroll: bool,
}

impl TextWriter {
    /// Log a message with prefix and color based on level
    pub fn log(&mut self, level: LogLevel, args: Arguments) {
        self.set_log_level_color(level);
        let _ = self.write_str(level.prefix());
        let _ = self.write_fmt(args);
        self.write_char('\n');
    }

    pub fn write_str(&mut self, s: &str) {
        for c in s.chars() {
            self.write_char(c);
        }
    }

    pub fn write_char(&mut self, c: char) {
        if c == '\n' {
            self.cursor_x = 0;
            self.cursor_y += self.line_height;
            if self.cursor_y + self.line_height >= self.height {
                if self.enable_scroll {
                    scroll_up(
                        self.framebuffer,
                        self.width,
                        self.height,
                        self.line_height,
                        self.bg_color,
                    );
                    self.cursor_y = self.height - self.line_height;
                } else {
                    self.cursor_y = self.height - self.line_height;
                }
            }
            return;
        }

        if let Some(glyph) = get_glyph(c) {
            draw_glyph(
                &glyph,
                self.fg_color,
                self.bg_color,
                self.framebuffer,
                self.width,
                self.height,
                self.cursor_x,
                self.cursor_y,
            );

            self.cursor_x += glyph.width() + 1;
            if self.cursor_x + glyph.width() >= self.width {
                self.cursor_x = 0;
                self.cursor_y += self.line_height;
                if self.cursor_y + self.line_height >= self.height {
                    if self.enable_scroll {
                        scroll_up(
                            self.framebuffer,
                            self.width,
                            self.height,
                            self.line_height,
                            self.bg_color,
                        );
                        self.cursor_y = self.height - self.line_height;
                    } else {
                        self.cursor_y = self.height - self.line_height;
                    }
                }
            }
        }
    }

    pub fn set_color(&mut self, fg: (u8, u8, u8), bg: (u8, u8, u8)) {
        self.fg_color = fg;
        self.bg_color = bg;
    }

    pub fn set_log_level_color(&mut self, level: LogLevel) {
        match level {
            LogLevel::Error => self.set_color((255, 0, 0), (0, 0, 0)),     // red
            LogLevel::Warn  => self.set_color((255, 255, 0), (0, 0, 0)),   // yellow
            LogLevel::Info  => self.set_color((0, 255, 0), (0, 0, 0)),     // green
            LogLevel::Debug => self.set_color((0, 0, 255), (0, 0, 0)),     // blue
            LogLevel::Trace => self.set_color((128, 128, 128), (0, 0, 0)), // gray
        }
    }
}

impl Write for TextWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_str(s);
        Ok(())
    }
}

// Global writer instance
lazy_static::lazy_static! {
    pub static ref WRITER: Mutex<Option<TextWriter>> = Mutex::new(None);
}

pub fn framebuffer_init(fb: &mut KernelFramebuffer) {
    let pixel_count = (fb.pitch * fb.height) / 4;
    let framebuffer = unsafe {
        slice::from_raw_parts_mut(fb.ptr as *mut u32, pixel_count)
    };

    let writer = TextWriter {
        fg_color: (255, 255, 255),
        bg_color: (0, 0, 0),
        cursor_x: 0,
        cursor_y: 0,
        width: fb.width,
        height: fb.height,
        line_height: get_glyph('A').map(|g| g.height() + 1).unwrap_or(16),
        framebuffer,
        enable_scroll: true,
    };

    *WRITER.lock() = Some(writer);
}

fn rgb((r, g, b): (u8, u8, u8)) -> u32 {
    ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
}

fn draw_glyph(
    glyph: &RasterizedChar,
    fg: (u8, u8, u8),
    bg: (u8, u8, u8),
    framebuffer: &mut [u32],
    screen_width: usize,
    screen_height: usize,
    cursor_x: usize,
    cursor_y: usize,
) {
    let width = glyph.width();
    let height = glyph.height();
    let bitmap = glyph.raster();

    for row in 0..height {
        for col in 0..width {
            let pixel = bitmap[row][col];
            let x = cursor_x + col;
            let y = cursor_y + row;

            if x < screen_width && y < screen_height {
                let fb_idx = y * screen_width + x;
                framebuffer[fb_idx] = if pixel > 0 {
                    rgb(fg)
                } else {
                    rgb(bg)
                };
            }
        }
    }
}

/// Scroll the framebuffer up by `line_height` pixels
fn scroll_up(framebuffer: &mut [u32], screen_width: usize, screen_height: usize, line_height: usize, bg: (u8,u8,u8)) {
    let row_pixels = line_height * screen_width;
    let total_pixels = screen_width * screen_height;

    framebuffer.copy_within(row_pixels..total_pixels, 0);

    // Clear bottom rows
    for y in (screen_height - line_height)..screen_height {
        for x in 0..screen_width {
            framebuffer[y * screen_width + x] = rgb(bg);
        }
    }
}
