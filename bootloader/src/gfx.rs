use uefi::boot::{self, SearchType};
use uefi::proto::console::gop::GraphicsOutput;
use uefi::Identify;
use uefi::prelude::*;

pub fn init_and_clear_blue(st: &uefi::SystemTable<uefi::Boot>) {
    let mut stdout = st.stdout();
    stdout.write_str("GOP: locating handles...\n").unwrap();

    let bt = st.boot_services();

    let handles = bt
        .locate_handle_buffer(SearchType::ByProtocol(&GraphicsOutput::GUID))
        .expect("locate_handle_buffer failed")
        .handles();

    stdout.write_str("GOP: found handle(s)\n").unwrap();

    let handle = handles[0];

    let gop = unsafe {
        bt.open_protocol::<GraphicsOutput>(
            boot::OpenProtocolParams {
                handle,
                agent: handle,
                controller: None,
            },
            boot::OpenProtocolAttributes::GetProtocol,
        )
    }
    .expect("open_protocol failed");

    let gop = gop.interface();

    let mode = gop.current_mode();
    let info = mode.info();
    let (width, height) = info.resolution();
    let stride = info.stride();

    let mut fb = gop.frame_buffer();
    let fb_bytes = unsafe { fb.as_mut_slice() };

    for y in 0..height {
        for x in 0..width {
            let idx = (y * stride + x) * 4;
            fb_bytes[idx + 0] = 0xFF;
            fb_bytes[idx + 1] = 0x00;
            fb_bytes[idx + 2] = 0x00;
            fb_bytes[idx + 3] = 0x00;
        }
    }

    stdout.write_str("GOP: framebuffer filled blue\n").unwrap();
}
