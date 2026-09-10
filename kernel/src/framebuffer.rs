// kernel/src/framebuffer.rs

use crate::color::Color;
use crate::serial_print;
use boot_proto::{BootInfo as ProtoBootInfo};
use x86_64::{PhysAddr, VirtAddr};

#[repr(C)]
pub struct KernelFramebuffer {
    pub ptr: *mut u8,
    pub width: usize,
    pub height: usize,
    pub pitch: usize,
    pub pixel_format: PixelFormat,
}

#[repr(u32)]
#[derive(Clone, Copy, Debug)]
pub enum PixelFormat {
    Rgb = 0,
    Bgr = 1,
    Bitmask = 2,
    BltOnly = 3,
}

impl KernelFramebuffer {
    pub fn from_bootinfo(boot_info: &ProtoBootInfo) -> Self {
        let fb = &boot_info.framebuffer;

        let mut fbk = Self {
            ptr: boot_info.framebuffer_virt as *mut u8,
            width: fb.width as usize,
            height: fb.height as usize,
            pitch: (fb.stride as usize) * 4,
            // temporary; will be overwritten by probe
            pixel_format: PixelFormat::Rgb,
        };

        // Detect real hardware format
        let detected = fbk.detect_real_pixel_format();
        fbk.pixel_format = detected;

        serial_print("Detected pixel_format: ");
        match detected {
            PixelFormat::Rgb => serial_print("Rgb\n"),
            PixelFormat::Bgr => serial_print("Bgr\n"),
            PixelFormat::Bitmask => serial_print("Bitmask\n"),
            PixelFormat::BltOnly => serial_print("BltOnly\n"),
        }

        fbk
    }

    #[inline(always)]
    pub fn put_pixel_color(&mut self, x: usize, y: usize, c: Color) {
        let color = self.pack_color(c.r, c.g, c.b);
        self.put_pixel(x, y, color);
    }

    #[inline(always)]
    pub fn draw_pixel_color(&mut self, x: usize, y: usize, c: Color) {
        let color = self.pack_color(c.r, c.g, c.b);
        self.draw_pixel(x, y, color);
    }

    pub fn fill_rect_color(&mut self, x: usize, y: usize, w: usize, h: usize, c: Color) {
        let color = self.pack_color(c.r, c.g, c.b);
        self.fill_rect(x, y, w, h, color);
    }

    pub fn clear_color(&mut self, c: Color) {
        let packed = self.pack_color(c.r, c.g, c.b);
        self.clear_fast(packed);
    }

    #[inline(always)]
    pub fn pack_color(&self, r: u8, g: u8, b: u8) -> u32 {
        match self.pixel_format {
            PixelFormat::Rgb =>
                ((r as u32) << 16) | ((g as u32) << 8) | (b as u32),

            PixelFormat::Bgr =>
                ((b as u32) << 16) | ((g as u32) << 8) | (r as u32),

            PixelFormat::Bitmask |
            PixelFormat::BltOnly =>
                // safest default for now
                ((r as u32) << 16) | ((g as u32) << 8) | (b as u32),
        }
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

    pub fn detect_real_pixel_format(&mut self) -> PixelFormat {
        let test_x = 0usize;
        let test_y = 0usize;

        let stride_pixels = self.pitch / 4;
        let idx = test_y * stride_pixels + test_x;
        let pixel_ptr = self.ptr as *mut u32;

        let original = unsafe { pixel_ptr.add(idx).read_volatile() };

        // Write BLUE using RGB packing
        let rgb_blue = (0x00 << 16) | (0x00 << 8) | 0xFF;
        unsafe { pixel_ptr.add(idx).write_volatile(rgb_blue); }

        let readback = unsafe { pixel_ptr.add(idx).read_volatile() };

        unsafe { pixel_ptr.add(idx).write_volatile(original); }

        let r = (readback >> 16) & 0xFF;
        let g = (readback >> 8) & 0xFF;
        let b = readback & 0xFF;

        if r == 0 && g == 0 && b == 255 {
            PixelFormat::Rgb
        } else if r == 255 && g == 0 && b == 0 {
            PixelFormat::Bgr
        } else {
            PixelFormat::Rgb
        }
    }
}






