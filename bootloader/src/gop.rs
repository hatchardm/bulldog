use core::slice;
use uefi::boot::{self, SearchType, ScopedProtocol};
use uefi::proto::console::gop::GraphicsOutput;
use uefi::prelude::*;
use uefi::Identify;

pub struct GopContext {
    pub scoped: ScopedProtocol<GraphicsOutput>,
    pub fb: &'static mut [u8],
    pub width: usize,
    pub height: usize,
    pub stride: usize,
}

impl GopContext {
    pub fn fill_color(&mut self, r: u8, g: u8, b: u8) {
        for y in 0..self.height {
            for x in 0..self.width {
                let idx = (y * self.stride + x) * 4;
                self.fb[idx + 0] = b;
                self.fb[idx + 1] = g;
                self.fb[idx + 2] = r;
                self.fb[idx + 3] = 0x00;
            }
        }
    }
}

pub fn init() -> Result<GopContext, Status> {
    let handle_buffer =
        boot::locate_handle_buffer(SearchType::ByProtocol(&GraphicsOutput::GUID))
            .map_err(|e| e.status())?;

    let handle = handle_buffer[0];

    let mut scoped: ScopedProtocol<GraphicsOutput> = unsafe {
        boot::open_protocol::<GraphicsOutput>(
            boot::OpenProtocolParams {
                handle,
                agent: handle,
                controller: None,
            },
            boot::OpenProtocolAttributes::GetProtocol,
        )
    }
    .map_err(|e| e.status())?;

    let gop: &mut GraphicsOutput = &mut *scoped;

    let info = gop.current_mode_info();
    let (width, height) = info.resolution();
    let stride = info.stride();

    let mut fb = gop.frame_buffer();
    let fb_ptr = fb.as_mut_ptr();
    let fb_len = (height * stride * 4) as usize;
    let fb_bytes = unsafe { slice::from_raw_parts_mut(fb_ptr, fb_len) };

    Ok(GopContext {
        scoped,
        fb: fb_bytes,
        width: width as usize,
        height: height as usize,
        stride: stride as usize,
    })
}



