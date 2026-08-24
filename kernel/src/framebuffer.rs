// kernel/src/framebuffer.rs

use crate::color::Color;
use boot_proto::{BootInfo as ProtoBootInfo, PixelFormat as ProtoPixelFormat};
use x86_64::{PhysAddr, VirtAddr};

#[repr(C)]
pub struct KernelFramebuffer {
    /// Virtual pointer (HHDM) to framebuffer memory.
    pub ptr: *mut u8,
    /// Visible width in pixels.
    pub width: usize,
    /// Visible height in pixels.
    pub height: usize,
    /// Bytes per row (stride × bytes_per_pixel).
    pub pitch: usize,
    /// Pixel format (RGB/BGR/etc).
    pub pixel_format: PixelFormat,
}

#[derive(Clone, Copy, Debug)]
pub enum PixelFormat {
    Rgb,
    Bgr,
    Bitmask,
    BltOnly,
}

impl KernelFramebuffer {
pub fn from_bootinfo(boot_info: &ProtoBootInfo) -> Self {
    let fb_virt = boot_info.physical_memory_offset + boot_info.framebuffer.addr;
    let fb = &boot_info.framebuffer;

    Self {
        ptr: fb_virt as *mut u8,                 // FIXED
        width: fb.width as usize,
        height: fb.height as usize,
        pitch: (fb.stride as usize) * 4,         // FIXED
        pixel_format: match fb.pixel_format {
            ProtoPixelFormat::Rgb => PixelFormat::Rgb,
            ProtoPixelFormat::Bgr => PixelFormat::Bgr,
            ProtoPixelFormat::Bitmask => PixelFormat::Bitmask,
            ProtoPixelFormat::BltOnly => PixelFormat::BltOnly,
        },
    }
}




    #[inline(always)]
    pub fn pack_color(&self, r: u8, g: u8, b: u8) -> u32 {
        match self.pixel_format {
            PixelFormat::Rgb => ((r as u32) << 16) | ((g as u32) << 8) | (b as u32),
            PixelFormat::Bgr => ((b as u32) << 16) | ((g as u32) << 8) | (r as u32),
            PixelFormat::Bitmask => ((b as u32) << 16) | ((g as u32) << 8) | (r as u32),
            PixelFormat::BltOnly => ((b as u32) << 16) | ((g as u32) << 8) | (r as u32),
        }
    }

    #[inline(always)]
    pub fn pack_color_c(&self, c: Color) -> u32 {
        self.pack_color(c.r, c.g, c.b)
    }

    pub fn clear_fast(&mut self, color: u32) {
        let stride_pixels = self.pitch / 4;
        let total_pixels = stride_pixels * self.height;
        let pixel_ptr = self.ptr as *mut u32;

        for i in 0..total_pixels {
            unsafe { pixel_ptr.add(i).write_volatile(color); }
        }
    }

    pub fn draw_pixel(&mut self, x: usize, y: usize, color: u32) {
        if x >= self.width || y >= self.height {
            return;
        }

        let stride_pixels = self.pitch / 4;
        let idx = y * stride_pixels + x;
        let pixel_ptr = self.ptr as *mut u32;

        unsafe { pixel_ptr.add(idx).write_volatile(color); }
    }

    pub fn fill_rect(&mut self, x: usize, y: usize, w: usize, h: usize, color: u32) {
        let max_x = (x + w).min(self.width);
        let max_y = (y + h).min(self.height);
        let stride_pixels = self.pitch / 4;
        let pixel_ptr = self.ptr as *mut u32;

        unsafe {
            for yy in y..max_y {
                let row = pixel_ptr.add(yy * stride_pixels);
                for xx in x..max_x {
                    row.add(xx).write_volatile(color);
                }
            }
        }
    }

    #[inline(always)]
    pub fn put_pixel(&mut self, x: usize, y: usize, color: u32) {
        if x >= self.width || y >= self.height {
            return;
        }

        let bytes_per_pixel = 4;
        let row_start = y * self.pitch;
        let offset = row_start + x * bytes_per_pixel;

        unsafe {
            let ptr = self.ptr.add(offset) as *mut u32;
            core::ptr::write_volatile(ptr, color);
        }
    }
}






