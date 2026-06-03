//! Framebuffer abstraction for the Bulldog kernel.
use crate::font::{get_glyph, FONT8X8};
use crate::serial::{serial_print, serial_print_u64};
use boot_proto::{BootInfo as ProtoBootInfo, PixelFormat as ProtoPixelFormat};
use x86_64::VirtAddr;



/// Lightweight framebuffer info extracted from `BootInfo`.
#[repr(C)]
pub struct FbInfo {
    /// Raw pointer to framebuffer memory (physical).
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
    /// Pixel format (RGB/BGR).
    pub pixel_format: PixelFormat,
}

/// Kernel-side pixel format enum.
#[derive(Clone, Copy, Debug)]
pub enum PixelFormat {
    Rgb,
    Bgr,
    Bitmask,
    BltOnly,
}

impl KernelFramebuffer {
pub fn from_bootinfo(boot: &ProtoBootInfo, phys_mem_offset: VirtAddr) -> Self {
    let fb = &boot.framebuffer;

    Self {
        ptr: (fb.addr + phys_mem_offset.as_u64()) as *mut u8,
        width: fb.width as usize,
        height: fb.height as usize,
        pitch: fb.stride as usize * 4,
        pixel_format: match fb.pixel_format {
            ProtoPixelFormat::Rgb => PixelFormat::Rgb,
            ProtoPixelFormat::Bgr => PixelFormat::Bgr,
            ProtoPixelFormat::Bitmask => PixelFormat::Bitmask,
            ProtoPixelFormat::BltOnly => PixelFormat::BltOnly,
        },
    }
}




pub fn pack_color(&self, r: u8, g: u8, b: u8) -> u32 {
    match self.pixel_format {
        PixelFormat::Rgb => {
            // R G B 0
            ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
        }
        PixelFormat::Bgr => {
            // B G R 0
            ((b as u32) << 16) | ((g as u32) << 8) | (r as u32)
        }
        PixelFormat::Bitmask => {
            // UEFI Bitmask is always BGRA 8:8:8:8
            ((b as u32) << 16) | ((g as u32) << 8) | (r as u32)
        }
        PixelFormat::BltOnly => {
            // BLT-only modes behave like BGR for raw writes
            ((b as u32) << 16) | ((g as u32) << 8) | (r as u32)
        }
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

    #[inline(always)]
    pub fn put_pixel(&mut self, x: usize, y: usize, color: u32) {
        self.draw_pixel(x, y, color);
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


pub fn draw_char_block_test(&mut self, x: usize, y: usize, fg: u32) {
    crate::serial::serial_print("draw_char_block_test: entered\n");
    self.draw_char_8x8_block(x, y, fg);
    crate::serial::serial_print("draw_char_block_test: leaving\n");
}



pub fn draw_char_8x8_block(&mut self, x: usize, y: usize, fg: u32) {
    for row in 0..8 {
        for col in 0..8 {
            self.draw_pixel(x + col, y + row, fg);
        }
    }
}







}


/// Extract framebuffer info from `BootInfo`.
pub fn boot_fb_info(boot_info: &boot_proto::BootInfo) -> Option<FbInfo> {
    if boot_info.framebuffer_present == 0 {
        return None;
    }

    let fb = &boot_info.framebuffer;
    let bytes_per_pixel: u64 = 4;

    Some(FbInfo {
        buffer_ptr: fb.addr,
        size_bytes: fb.stride as u64 * fb.height as u64 * bytes_per_pixel,
        width: fb.width as usize,
        height: fb.height as usize,
        pitch: fb.stride as usize * 4,
    })
}








