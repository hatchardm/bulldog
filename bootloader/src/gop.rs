use core::slice;
use uefi::boot::{self, ScopedProtocol};
use uefi::proto::console::gop::{GraphicsOutput, ModeInfo};
use uefi::prelude::*;

use crate::framebuffer::Framebuffer;

pub struct GopContext<'a> {
    pub gop: ScopedProtocol<GraphicsOutput>,
    pub fb: Framebuffer<'a>,
    pub mode: ModeInfo,
}

pub fn init() -> Result<GopContext<'static>, Status> {
    let handle = boot::get_handle_for_protocol::<GraphicsOutput>()
        .map_err(|e: uefi::Error| e.status())?;

    let mut gop = boot::open_protocol_exclusive::<GraphicsOutput>(handle)
        .map_err(|e: uefi::Error| e.status())?;

    let info = gop.current_mode_info().clone();
    let (width, height) = info.resolution();
    let stride = info.stride();

    let mut fb_raw = gop.frame_buffer();
    let fb_ptr = fb_raw.as_mut_ptr();
    let fb_len = (height * stride * 4) as usize;

    let fb_bytes = unsafe { slice::from_raw_parts_mut(fb_ptr, fb_len) };

    let fb = Framebuffer {
        data: fb_bytes,
        width: width as usize,
        height: height as usize,
        stride: stride as usize,
    };

    Ok(GopContext {
        gop,
        fb,
        mode: info,
    })
}





