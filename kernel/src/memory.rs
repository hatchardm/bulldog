use bootloader_api::info::{MemoryRegion, MemoryRegionKind};
use x86_64::{
    PhysAddr, VirtAddr,
    structures::paging::{
        FrameAllocator, Mapper, OffsetPageTable, Page, PageTable, PageTableFlags,
        PhysFrame, Size4KiB,
    },
    registers::control::Cr3,
};
use crate::{print, println};

extern crate alloc;
use alloc::vec::Vec;

/// Initialize a new OffsetPageTable.
///
/// # Safety
/// Caller must ensure the complete physical memory is mapped to virtual memory
/// at `physical_memory_offset`, and this is only called once.
pub unsafe fn init(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
    let level_4_table = active_level_4_table(physical_memory_offset);
    OffsetPageTable::new(level_4_table, physical_memory_offset)
}

/// Returns a mutable reference to the active level 4 table.
///
/// # Safety
/// Same safety requirements as `init`.
unsafe fn active_level_4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
    let (level_4_table_frame, _) = Cr3::read();
    let phys = level_4_table_frame.start_address();
    let virt = physical_memory_offset + phys.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();
    &mut *page_table_ptr
}

/// Initializes an OffsetPageTable using the given physical memory offset.
pub unsafe fn init_offset_page_table(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
    OffsetPageTable::new(active_level_4_table(physical_memory_offset), physical_memory_offset)
}

/// A FrameAllocator that always returns `None`.
pub struct EmptyFrameAllocator;

unsafe impl FrameAllocator<Size4KiB> for EmptyFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        None
    }
}

/// A FrameAllocator that returns usable frames from the bootloader's memory map.
pub struct BootInfoFrameAllocator {
    memory_map: &'static [MemoryRegion],
    frames: Vec<PhysFrame>,
    next: usize,
}

impl BootInfoFrameAllocator {
    /// Create a FrameAllocator from the passed memory map.
    ///
    /// # Safety
    /// Caller must ensure all `USABLE` frames are truly unused.
    pub unsafe fn init(memory_map: &'static [MemoryRegion]) -> Self {
        let frames = memory_map
            .iter()
            .filter(|r| r.kind == MemoryRegionKind::Usable)
            .flat_map(|r| (r.start..r.end).step_by(4096))
            .map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
            .filter(|frame| frame.start_address().as_u64() >= 0x10000)
            .collect();

        BootInfoFrameAllocator {
            memory_map,
            frames,
            next: 0,
        }
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        let frame = self.frames.get(self.next).cloned();
        self.next += 1;
        frame
    }
}

/// Maps the LAPIC MMIO region into the virtual address space.
pub fn map_lapic_mmio(
    mapper: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) {
    let lapic_phys = PhysAddr::new(0xfee00000);
    let lapic_virt = VirtAddr::new(0xfee00000);
    let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;

    map_page(mapper, lapic_virt, lapic_phys, flags, frame_allocator);
}

/// Maps a single page to a physical frame with the given flags.
pub fn map_page(
    mapper: &mut impl Mapper<Size4KiB>,
    virt: VirtAddr,
    phys: PhysAddr,
    flags: PageTableFlags,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) {
    let page = Page::containing_address(virt);
    let frame = PhysFrame::containing_address(phys);

    unsafe {
        mapper
            .map_to(page, frame, flags, frame_allocator)
            .expect("map_page failed")
            .flush();
    }
}



