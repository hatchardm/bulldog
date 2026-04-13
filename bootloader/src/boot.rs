// boot.rs

use crate::console::Console;
use uefi::system;
use uefi::table::cfg::ConfigTableEntry;


pub enum FramebufferFormat {
    Rgb,
    Bgr,
    Bitmask,
    Unknown,
}

pub struct BootInfo {
    pub fb_ptr: *mut u8,
    pub fb_width: usize,
    pub fb_height: usize,
    pub fb_stride: usize,
    pub fb_format: FramebufferFormat,
    pub rsdp_addr: usize,
}


pub fn load_kernel(console: &mut Console) -> Result<(), ()> {
    console.write_str("load_kernel(): stub\n");
    Ok(())
}

pub fn jump_to_kernel(_boot_info: &BootInfo) -> ! {
    loop {}
}

pub fn find_rsdp() -> Option<usize> {
    let mut rsdp = None;

    system::with_config_table(|entries| {
        for entry in entries {
            if entry.guid == ConfigTableEntry::ACPI2_GUID {
                rsdp = Some(entry.address as usize);
            }
        }
    });

    rsdp
}




   





