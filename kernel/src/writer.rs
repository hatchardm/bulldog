use core::fmt::{self, Write, Arguments};
use spin::Mutex;
use crate::framebuffer::KernelFramebuffer;
use crate::font::get_glyph;
use noto_sans_mono_bitmap::RasterizedChar;

/// Kernel log levels mapped to color-coded output.
#[derive(Copy, Clone)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl LogLevel {
    /// Returns the textual prefix associated with each log level.
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

/// TextWriter renders characters into the kernel framebuffer.
/// It tracks cursor position, colors, and handles scrolling.
pub struct TextWriter {
    pub fg_color: (u8, u8, u8),
    pub bg_color: (u8, u8, u8),
    pub cursor_x: usize,
    pub cursor_y: usize,
    pub width: usize,          // visible width in pixels
    pub height: usize,         // visible height in pixels
    pub line_height: usize,    // font height in pixels
    pub stride_pixels: usize,  // pixels per row (pitch / 4)
    pub framebuffer: &'static mut [u32],
    pub enable_scroll: bool,
}

impl TextWriter {
    /// Log a message with level prefix and color.
    pub fn log(&mut self, level: LogLevel, args: Arguments) {
        self.set_log_level_color(level);
        self.write_str_inner(level.prefix());
        let _ = self.write_fmt(args);
        self.write_char('\n');
    }

    /// Internal helper to write a string without recursion.
    pub fn write_str_inner(&mut self, s: &str) {
        for c in s.chars() {
            self.write_char(c);
        }
    }

    /// Write a single character to the framebuffer.
    /// Handles newline, scrolling, and glyph rendering.
   pub fn write_char(&mut self, c: char) {
    if c == '\n' {
        self.cursor_x = 0;
        self.cursor_y += self.line_height;
        if self.cursor_y + self.line_height >= self.height {
            if self.enable_scroll {
                scroll_up(
                    self.framebuffer,
                    self.stride_pixels,
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
            self.stride_pixels,
            self.height,
            self.cursor_x,
            self.cursor_y,
        );

        // spacing tweak: give digits extra breathing room
        if c.is_ascii_digit() {
            self.cursor_x += glyph.width() + 2;
        } else {
            self.cursor_x += glyph.width() + 1;
        }

        if self.cursor_x + glyph.width() >= self.width {
            self.cursor_x = 0;
            self.cursor_y += self.line_height;
            if self.cursor_y + self.line_height >= self.height {
                if self.enable_scroll {
                    scroll_up(
                        self.framebuffer,
                        self.stride_pixels,
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


    /// Set foreground and background colors.
    pub fn set_color(&mut self, fg: (u8, u8, u8), bg: (u8, u8, u8)) {
        self.fg_color = fg;
        self.bg_color = bg;
    }

    /// Set colors based on log level.
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

/// Implement fmt::Write so TextWriter can be used with `write!` macros.
impl fmt::Write for TextWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_str_inner(s);
        Ok(())
    }
}

/// Global writer instance protected by a spinlock.
/// Initialized during framebuffer setup.
lazy_static::lazy_static! {
    pub static ref WRITER: Mutex<Option<TextWriter>> = Mutex::new(None);
}

/// Initialize the global WRITER from a KernelFramebuffer.
/// Maps the framebuffer pointer into a slice and constructs TextWriter.
pub fn framebuffer_init(fb: &mut KernelFramebuffer) {
    let stride_pixels = fb.pitch / 4;
    let len = stride_pixels * fb.height;
    let framebuffer: &'static mut [u32] = unsafe {
        core::slice::from_raw_parts_mut(fb.ptr as *mut u32, len)
    };

    let writer = TextWriter {
        fg_color: (255, 255, 255),
        bg_color: (0, 0, 0),
        cursor_x: 0,
        cursor_y: 0,
        width: fb.width,
        height: fb.height,
        line_height: 16, // must match font height
        stride_pixels,
        framebuffer,
        enable_scroll: true,
    };

    WRITER.lock().replace(writer);
}

/// Scroll the framebuffer up by one line_height.
/// Shifts rows up and clears the bottom region.
pub fn scroll_up(
    framebuffer: &mut [u32],
    stride_pixels: usize,
    height: usize,
    line_height: usize,
    bg_color: (u8, u8, u8),
) {
    let bg = ((bg_color.0 as u32) << 16)
           | ((bg_color.1 as u32) << 8)
           | (bg_color.2 as u32);

    for y in 0..(height - line_height) {
        let dst = y * stride_pixels;
        let src = (y + line_height) * stride_pixels;
        framebuffer.copy_within(src..src + stride_pixels, dst);
    }

    for y in (height - line_height)..height {
        let row_start = y * stride_pixels;
        for x in 0..stride_pixels {
            framebuffer[row_start + x] = bg;
        }
    }
}

/// Draw a glyph into the framebuffer at (x,y).
/// Uses alpha channel from raster to decide between fg and bg color.
pub fn draw_glyph(
    glyph: &RasterizedChar,
    fg: (u8, u8, u8),
    bg: (u8, u8, u8),
    framebuffer: &mut [u32],
    stride_pixels: usize,
    height: usize,
    x: usize,
    y: usize,
) {
    let fg_color = (0xFF << 24)
             | ((fg.0 as u32) << 16)
             | ((fg.1 as u32) << 8)
             | (fg.2 as u32);

    let bg_color = (0xFF << 24)
             | ((bg.0 as u32) << 16)
             | ((bg.1 as u32) << 8)
             | (bg.2 as u32);


    let glyph_width = glyph.width();
    let glyph_height = glyph.height();
    let raster: &[&[u8]] = glyph.raster();

    for row in 0..glyph_height {
        if y + row >= height { break; }
        let row_start = (y + row) * stride_pixels;
        let row_data: &[u8] = raster[row];
        for col in 0..glyph_width {
            if x + col >= stride_pixels { break; }
            let idx = row_start + (x + col);
            let alpha: u8 = row_data[col];
            framebuffer[idx] = if alpha > 0 { fg_color } else { bg_color };
        }
    }
}





