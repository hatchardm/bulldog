#![no_std]

pub const MAX_MEMORY_REGIONS: usize = 128;

#[repr(C)]
pub struct BootInfo {
    pub framebuffer: Framebuffer,
    pub framebuffer_present: u8,
    pub _pad0: [u8; 7],
    pub physical_memory_offset: u64,
    pub memory_regions: *const MemoryRegion,
    pub memory_region_count: usize,
    pub memory_regions_buffer: [MemoryRegion; MAX_MEMORY_REGIONS],
    pub kernel_phys_start: u64,
    pub kernel_phys_end: u64,
    pub kernel_entry_phys: u64,
    pub framebuffer_virt: u64,

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
#[derive(Clone, Copy)]
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
