use core::slice;
use uefi::boot::{self, SearchType, ScopedProtocol};
use uefi::proto::console::gop::GraphicsOutput;
use uefi::prelude::*;
use uefi::Identify;
use crate::framebuffer::Framebuffer;
use crate::color::Color;


pub struct GopContext {
    pub scoped: ScopedProtocol<GraphicsOutput>,
    pub fb: Framebuffer<'static>,
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

    let fb = Framebuffer {
        data: fb_bytes,
        width: width as usize,
        height: height as usize,
        stride: stride as usize,
    };

    Ok(GopContext { scoped, fb })
}




