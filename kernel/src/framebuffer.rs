pub use bootloader_api::info::{FrameBuffer, PixelFormat};

pub struct KernelFramebuffer {
    pub ptr: *mut u8,
    pub width: usize,
    pub height: usize,
    pub pitch: usize,
    pub pixel_format: PixelFormat,
}

impl KernelFramebuffer {
    /// Construct from bootloader framebuffer
    pub fn from_bootloader(fb: &mut FrameBuffer) -> Self {
        let info = fb.info();
        Self {
            ptr: fb.buffer_mut().as_mut_ptr(),
            width: info.width,
            height: info.height,
            pitch: info.stride * info.bytes_per_pixel,
            pixel_format: info.pixel_format,
        }
    }

    pub fn pack_color(&self, r: u8, g: u8, b: u8) -> u32 {
    match self.pixel_format {
        PixelFormat::Rgb => ((r as u32) << 16) | ((g as u32) << 8) | (b as u32),
        PixelFormat::Bgr => ((b as u32) << 16) | ((g as u32) << 8) | (r as u32),
        _ => 0, // or panic!("Unsupported pixel format")
    }
}


    /// Fast screen clear using packed u32 color
    pub fn clear_fast(&mut self, color: u32) {
        let pixel_ptr = self.ptr as *mut u32;
        let total_pixels = self.pitch / 4 * self.height;

        for i in 0..total_pixels {
            unsafe {
                pixel_ptr.add(i).write_volatile(color);
            }
        }
    }

    /// Draw a single pixel at (x, y)
    pub fn draw_pixel(&mut self, x: usize, y: usize, color: u32) {
        if x >= self.width || y >= self.height {
            return;
        }

        let idx = y * self.pitch / 4 + x;
        let pixel_ptr = self.ptr as *mut u32;

        unsafe {
            pixel_ptr.add(idx).write_volatile(color);
        }
    }

    /// Draw a filled rectangle at (x, y) with width and height
    pub fn draw_rect(&mut self, x: usize, y: usize, w: usize, h: usize, color: u32) {
        for dy in 0..h {
            for dx in 0..w {
                self.draw_pixel(x + dx, y + dy, color);
            }
        }
    }
}




