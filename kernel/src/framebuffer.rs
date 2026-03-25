//! Framebuffer abstraction for the Bulldog kernel.
//!
//! Converts the boot-proto framebuffer into a KernelFramebuffer,
//! providing safe pixel operations and fast clearing.

use boot_proto::{Framebuffer as ProtoFramebuffer, PixelFormat as ProtoPixelFormat};

/// Lightweight framebuffer info extracted from `BootInfo`.
/// Used by subsystems that only need metadata.
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

/// KernelFramebuffer wraps the boot-proto framebuffer.
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

/// Kernel-side pixel format enum.
/// Mirrors the boot-proto pixel format.
#[derive(Clone, Copy, Debug)]
pub enum PixelFormat {
    Rgb,
    Bgr,
    Bitmask,
    BltOnly,
}

impl KernelFramebuffer {
    /// Construct a `KernelFramebuffer` from a `boot-proto` framebuffer.
    pub fn from_bulldog(fb: &mut ProtoFramebuffer) -> Self {
        // UEFI GOP always uses 32-bit pixels (4 bytes per pixel)
        let bytes_per_pixel = 4;

        Self {
            ptr: fb.addr,
            width: fb.width,
            height: fb.height,
            pitch: fb.stride * bytes_per_pixel,
            pixel_format: match fb.pixel_format {
                ProtoPixelFormat::Rgb => PixelFormat::Rgb,
                ProtoPixelFormat::Bgr => PixelFormat::Bgr,
                ProtoPixelFormat::Bitmask => PixelFormat::Bitmask,
                ProtoPixelFormat::BltOnly => PixelFormat::BltOnly,
            },
        }
    }

    /// Pack RGB values into a 32‑bit pixel according to format.
    pub fn pack_color(&self, r: u8, g: u8, b: u8) -> u32 {
        match self.pixel_format {
            PixelFormat::Rgb =>
                (0xFF << 24) | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32),

            PixelFormat::Bgr =>
                (0xFF << 24) | ((b as u32) << 16) | ((g as u32) << 8) | (r as u32),

            _ =>
                (0xFF << 24) | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32),
        }
    }

    /// Clear the entire framebuffer with a solid color.
    pub fn clear_fast(&mut self, color: u32) {
        let stride_pixels = self.pitch / 4;
        let total_pixels = stride_pixels * self.height;
        let pixel_ptr = self.ptr as *mut u32;

        for i in 0..total_pixels {
            unsafe { pixel_ptr.add(i).write_volatile(color); }
        }
    }

    /// Draw a single pixel at (x,y).
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
pub fn boot_fb_info(boot_info: &boot_proto::BootInfo) -> Option<FbInfo> {
    boot_info.framebuffer.as_ref().map(|fb| {
        let bytes_per_pixel = 4;

        FbInfo {
            buffer_ptr: fb.addr as u64,
            size_bytes: (fb.stride * fb.height * bytes_per_pixel) as u64,
            width: fb.width,
            height: fb.height,
            pitch: fb.stride * bytes_per_pixel,
        }
    })
}







