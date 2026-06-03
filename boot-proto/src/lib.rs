#![no_std]

#[repr(C)]
pub struct BootInfo {
    pub framebuffer: Framebuffer,      // always present
    pub framebuffer_present: u8,       // 0 = no fb, 1 = valid
    pub _pad0: [u8; 7],                // padding to 16‑byte align

    pub physical_memory_offset: u64,

    pub memory_regions: *const MemoryRegion,
    pub memory_region_count: usize,
}

#[repr(C)]
pub struct Framebuffer {
    pub addr: u64,          // physical address
    pub width: u32,
    pub height: u32,
    pub stride: u32,        // pixels per row
    pub pixel_format: PixelFormat,
}

#[repr(u32)]
#[derive(Clone, Copy, Debug)]
pub enum PixelFormat {
    Rgb = 0,
    Bgr = 1,
    Bitmask = 2,
    BltOnly = 3,
}

#[repr(C)]
pub struct MemoryRegion {
    pub start: u64,
    pub end: u64,
    pub kind: MemoryRegionKind,
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MemoryRegionKind {
    Usable = 0,
    Reserved = 1,
    Acpi = 2,
    Mmio = 3,
    Unknown = 4,
}
