use crate::Console;

pub struct BootInfo {
    pub fb_ptr: *mut u8,
    pub fb_width: usize,
    pub fb_height: usize,
    pub fb_stride: usize,
}


pub fn load_kernel(console: &mut Console) -> Result<(), ()> {
    console.write_str("load_kernel(): stub\n");
    Ok(())
}


pub fn jump_to_kernel(_boot_info: &BootInfo) -> ! {
    // For now, just halt
    loop {}
}
