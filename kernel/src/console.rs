use crate::framebuffer::KernelFramebuffer;
use crate::text::draw_char_8x8;

pub struct Console {
    fb: *mut KernelFramebuffer,
    cursor_x: usize,
    cursor_y: usize,
    fg: u32,
    bg: u32,
}

impl Console {
    pub fn new(fb: &mut KernelFramebuffer) -> Self {
        Console {
            fb: fb as *mut KernelFramebuffer,
            cursor_x: 0,
            cursor_y: 0,
            fg: 0x00FFFFFF,
            bg: 0x00000000,
        }
    }

     pub fn write_str(&mut self, _s: &str) {
    let fb: &mut KernelFramebuffer = unsafe { &mut *self.fb };

    fb.draw_pixel(0, 0, self.fg);
}

}




    
/* 
    pub fn write_str(&mut self, s: &str) {
        let fb = unsafe { &mut *self.fb };

        draw_char_8x8(fb, 0, 0, 'X', self.fg, self.bg);
    }
    
}
*/


