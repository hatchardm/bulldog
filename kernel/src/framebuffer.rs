use bootloader_api::info::{BootInfo, FrameBuffer, PixelFormat};

/// Lightweight info struct for binding the writer
pub struct FbInfo {
    pub buffer_ptr: u64,
    pub size_bytes: u64,
    pub width: usize,
    pub height: usize,
    pub pitch: usize, // bytes per row
}

/// KernelFramebuffer wraps the bootloader framebuffer
pub struct KernelFramebuffer {
    pub ptr: *mut u8,
    pub width: usize,
    pub height: usize,
    pub pitch: usize, // bytes per row
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

    /// Pack RGB values into a u32 pixel according to format
    pub fn pack_color(&self, r: u8, g: u8, b: u8) -> u32 {
        match self.pixel_format {
            PixelFormat::Rgb => ((r as u32) << 16) | ((g as u32) << 8) | (b as u32),
            PixelFormat::Bgr => ((b as u32) << 16) | ((g as u32) << 8) | (r as u32),
            _ => ((r as u32) << 16) | ((g as u32) << 8) | (b as u32),
        }
    }

    /// Clear the entire framebuffer with a solid color
    pub fn clear_fast(&mut self, color: u32) {
        let total_pixels = (self.pitch / 4) * self.height;
        let pixel_ptr = self.ptr as *mut u32;
        for i in 0..total_pixels {
            unsafe { pixel_ptr.add(i).write_volatile(color); }
        }
    }

    /// Draw a single pixel
    pub fn draw_pixel(&mut self, x: usize, y: usize, color: u32) {
        if x >= self.width || y >= self.height {
            return;
        }
        let idx = y * (self.pitch / 4) + x;
        let pixel_ptr = self.ptr as *mut u32;
        unsafe { pixel_ptr.add(idx).write_volatile(color); }
    }
}

/// Extract framebuffer info from BootInfo
pub fn boot_fb_info(boot_info: &BootInfo) -> Option<FbInfo> {
    boot_info.framebuffer.as_ref().map(|fb| {
        let info = fb.info();
        FbInfo {
            buffer_ptr: fb.buffer().as_ptr() as u64,
            size_bytes: (info.stride * info.height * info.bytes_per_pixel) as u64,
            width: info.width,
            height: info.height,
            pitch: info.stride * info.bytes_per_pixel,
        }
    })
}




