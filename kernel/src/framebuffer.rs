use bootloader_api::info::{BootInfo, FrameBuffer, PixelFormat};

/// Lightweight framebuffer info extracted from `BootInfo`.
/// Used to bind the writer without holding a full `FrameBuffer`.
pub struct FbInfo {
    /// Raw pointer to framebuffer memory.
    pub buffer_ptr: u64,
    /// Total size of framebuffer in bytes.
    pub size_bytes: u64,
    /// Visible width in pixels.
    pub width: usize,
    /// Visible height in pixels.
    pub height: usize,
    /// Bytes per row (stride × bytes_per_pixel).
    pub pitch: usize,
}

/// KernelFramebuffer wraps the bootloader framebuffer.
/// Provides safe abstractions for pixel operations.
pub struct KernelFramebuffer {
    /// Raw pointer to framebuffer memory.
    pub ptr: *mut u8,
    /// Visible width in pixels.
    pub width: usize,
    /// Visible height in pixels.
    pub height: usize,
    /// Bytes per row (stride × bytes_per_pixel).
    pub pitch: usize,
    /// Pixel format (RGB/BGR).
    pub pixel_format: PixelFormat,
}

impl KernelFramebuffer {
    /// Construct a `KernelFramebuffer` from bootloader framebuffer.
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

    /// Pack RGB values into a 32‑bit pixel according to format.
    /// Supports RGB and BGR layouts; defaults to RGB otherwise.
   pub fn pack_color(&self, r: u8, g: u8, b: u8) -> u32 {
    match self.pixel_format {
        PixelFormat::Rgb => (0xFF << 24) | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32),
        PixelFormat::Bgr => (0xFF << 24) | ((b as u32) << 16) | ((g as u32) << 8) | (r as u32),
        _ => (0xFF << 24) | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32),
    }
}


    /// Clear the entire framebuffer with a solid color.
    /// Uses volatile writes to ensure memory‑mapped I/O is respected.
    pub fn clear_fast(&mut self, color: u32) {
        let stride_pixels = self.pitch / 4;
        let total_pixels = stride_pixels * self.height;
        let pixel_ptr = self.ptr as *mut u32;
        for i in 0..total_pixels {
            unsafe { pixel_ptr.add(i).write_volatile(color); }
        }
    }

    /// Draw a single pixel at (x,y).
    /// Bounds‑checked to avoid writing outside framebuffer.
    pub fn draw_pixel(&mut self, x: usize, y: usize, color: u32) {
        if x >= self.width || y >= self.height {
            return;
        }
        let stride_pixels = self.pitch / 4;
        let idx = y * stride_pixels + x;
        let pixel_ptr = self.ptr as *mut u32;
        unsafe { pixel_ptr.add(idx).write_volatile(color); }
    }
}

/// Extract framebuffer info from `BootInfo`.
/// Returns `None` if no framebuffer is present.
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






