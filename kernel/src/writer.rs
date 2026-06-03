use core::fmt::{self, Write};
use crate::framebuffer::KernelFramebuffer;
use crate::font::FONT8X8;

// static mut WRITER_PTR: *mut TextWriter = core::ptr::null_mut();

pub struct TextWriter {
    pub fg: (u8, u8, u8),
    pub bg: (u8, u8, u8),
    pub x: usize,
    pub y: usize,
    pub width: usize,
    pub height: usize,
    pub stride: usize,
    pub fb: &'static mut [u32],
}

impl TextWriter {
    pub fn write_char(&mut self, c: char) {
        if c == '\n' {
            self.x = 0;
            self.y += 8;
            return;
        }

        if c < ' ' || c > '~' {
            return;
        }

        if self.x + 8 >= self.width {
            self.x = 0;
            self.y += 8;
        }

        // 🔥 TEMP: disable drawing
        /*
        let idx = (c as usize) - 0x20;
        let glyph = &FONT8X8[idx];

        draw_8x8(
            self.fb,
            self.stride,
            self.width,
            self.height,
            self.x,
            self.y,
            glyph,
            self.fg,
            self.bg,
        );
        */

        self.x += 8;
    }

      pub fn write_str_inner(&mut self, s: &str) {
        for b in s.bytes() {
            self.write_char(b as char);
        }
    }
}


impl Write for TextWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_str_inner(s);
        Ok(())
    }
}

pub fn framebuffer_init(fb: &mut KernelFramebuffer, writer: &mut TextWriter) {
    let stride = fb.pitch / 4;
    let len = stride * fb.height;

    let fb_slice: &'static mut [u32] = unsafe {
        core::slice::from_raw_parts_mut(fb.ptr as *mut u32, len)
    };

    // 🔥 TEST: draw a few obvious pixels
    if len > 0 {
        fb_slice[0] = 0xFFFF0000; // top-left red
    }
    if len > stride + 1 {
        fb_slice[stride + 1] = 0xFF00FF00; // one pixel down-right green
    }

    writer.fg = (255, 255, 255);
    writer.bg = (0, 0, 0);
    writer.x = 0;
    writer.y = 0;
    writer.width = fb.width;
    writer.height = fb.height;
    writer.stride = stride;
    writer.fb = fb_slice;
/* 
    unsafe {
        WRITER_PTR = writer as *mut TextWriter;
    }  */
}


// pub fn kprint(s: &str) { ... unchanged but commented out }

pub fn draw_8x8(
    fb: &mut [u32],
    stride: usize,
    width: usize,
    height: usize,
    x: usize,
    y: usize,
    glyph: &[u8; 8],
    fg: (u8, u8, u8),
    bg: (u8, u8, u8),
) {
    let fg_color = 0xFF000000
        | ((fg.0 as u32) << 16)
        | ((fg.1 as u32) << 8)
        | (fg.2 as u32);

    let bg_color = 0xFF000000
        | ((bg.0 as u32) << 16)
        | ((bg.1 as u32) << 8)
        | (bg.2 as u32);

    for row in 0..8 {
        if y + row >= height {
            break;
        }
        let bits = glyph[row];
        let row_start = (y + row) * stride;

        for col in 0..8 {
            if x + col >= width {
                break;
            }
            let idx = row_start + (x + col);
            let bit = (bits << col) & 0x80;
            fb[idx] = if bit != 0 { fg_color } else { bg_color };
        }
    }
}



