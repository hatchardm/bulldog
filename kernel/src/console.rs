// console.rs
use crate::framebuffer::KernelFramebuffer;
use crate::serial::serial_print;

pub struct Console {
    fb: KernelFramebuffer,
    cursor_x: usize,
    cursor_y: usize,
    fg: u32,
    bg: u32,
}

impl Console {
    pub fn new(fb: KernelFramebuffer) -> Self {
        Console {
            fb,
            cursor_x: 0,
            cursor_y: 0,
            fg: 0x00FF_FFFF,
            bg: 0x0000_0000,
        }
    }

    pub fn write_str(&mut self, s: &str) {

        for b in s.bytes() {
            match b {
                b'\n' => {
                    // TODO: implement newline handling
                }
                _ => {
                    // TODO: draw a glyph for this ASCII byte using self.fb
                }
            }
        }
    }
}



