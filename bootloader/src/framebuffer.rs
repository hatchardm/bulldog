use uefi::proto::console::gop::FrameBuffer;

pub struct Framebuffer {
    pub ptr: *mut u8,
    pub size: usize,
    pub width: usize,
    pub height: usize,
    pub stride: usize, // pixels per scanline
}

impl Framebuffer {
    pub fn new(fb: &mut FrameBuffer, width: usize, height: usize, stride: usize) -> Self {
        // stride is in pixels → convert to bytes
        let size = height * stride * 4;

        Framebuffer {
            ptr: fb.as_mut_ptr(),
            width,
            height,
            stride,
            size,
        }
    }

    #[inline]
    pub fn put_pixel(&mut self, x: usize, y: usize, color: crate::color::Color) {
        if x >= self.width || y >= self.height {
            return;
        }

        // Correct offset: (y * stride + x) * 4
        let offset = (y * self.stride + x) * 4;

        unsafe {
            let p = self.ptr.add(offset);
            *p.add(0) = color.b; // BGRX layout (most common under OVMF)
            *p.add(1) = color.g;
            *p.add(2) = color.r;
            *p.add(3) = 0;
        }
    }

    pub fn clear(&mut self) {
        unsafe {
            core::ptr::write_bytes(self.ptr, 0, self.size);
        }
    }
}

