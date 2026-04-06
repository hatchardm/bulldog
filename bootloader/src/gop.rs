use core::slice;
use uefi::boot;
use uefi::proto::console::gop::GraphicsOutput;
use uefi::prelude::*;
use uefi::boot::ScopedProtocol;



use crate::framebuffer::Framebuffer;

pub struct GopContext<'a> {
    pub gop: ScopedProtocol<GraphicsOutput>,
    pub fb: Framebuffer<'a>,
}

pub fn init() -> Result<GopContext<'static>, Status> {
    // 1. Get handle for GOP
    let handle = boot::get_handle_for_protocol::<GraphicsOutput>()
        .map_err(|e| e.status())?;

    // 2. Open GOP exclusively (this returns GraphicsOutput, NOT ScopedProtocol)
    let mut gop = boot::open_protocol_exclusive::<GraphicsOutput>(handle)
        .map_err(|e| e.status())?;

    // 3. Get mode info
    let info = gop.current_mode_info();
    let (width, height) = info.resolution();
    let stride = info.stride();

    // 4. Get framebuffer pointer + size
    let mut fb_raw = gop.frame_buffer();
    let fb_ptr = fb_raw.as_mut_ptr();
    let fb_len = (height * stride * 4) as usize;

    // 5. Convert to slice
    let fb_bytes = unsafe { slice::from_raw_parts_mut(fb_ptr, fb_len) };

    // 6. Build your framebuffer struct
    let fb = Framebuffer {
        data: fb_bytes,
        width: width as usize,
        height: height as usize,
        stride: stride as usize,
    };

    // 7. Return context
    Ok(GopContext { gop, fb })
}





