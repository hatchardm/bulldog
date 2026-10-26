//! Simple RAM-backed block device for FAT32
//!
//! This exposes a read-only block device backed by a physical
//! memory region containing the FAT disk image loaded by the bootloader.

use core::ptr;

/// A simple RAM disk backed by a physical memory region.
/// The bootloader loads `fat.img` into RAM and passes its
/// physical address + size via BootInfo.
pub struct RamDisk {
    pub phys_addr: u64,
    pub size: u64,
}

impl RamDisk {
    /// Read a 512-byte sector from the RAM disk.
    ///
    /// `lba` is the sector index (0-based).
    /// `buf` must be exactly 512 bytes.
    pub fn read_sector(&self, lba: u64, buf: &mut [u8; 512]) {
        let offset = lba * 512;

        // Out-of-bounds read → return zeros
        if offset + 512 > self.size {
            buf.fill(0);
            return;
        }

        let src = (self.phys_addr + offset) as *const u8;

        unsafe {
            ptr::copy_nonoverlapping(src, buf.as_mut_ptr(), 512);
        }
    }
}
