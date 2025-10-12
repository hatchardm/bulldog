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

pub struct PreHeapAllocator {
    pub memory_map: &'static [MemoryRegion],
    pub frames: [Option<PhysFrame>; 512],
    pub next: usize,
}

pub struct BootInfoFrameAllocator {
    pub memory_map: &'static [MemoryRegion],
    pub frames: Vec<PhysFrame>,
    pub next: usize,
}

impl PreHeapAllocator {
    pub fn into_vec(self) -> Vec<PhysFrame> {
        self.frames.iter().filter_map(|&f| f).collect()
    }
}


unsafe impl FrameAllocator<Size4KiB> for PreHeapAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        while self.next < self.frames.len() {
            if let Some(frame) = self.frames[self.next].take() {
                self.next += 1;
                return Some(frame);
            }
            self.next += 1;
        }
        None
    }
}


impl BootInfoFrameAllocator {
    pub fn new(memory_map: &'static [MemoryRegion], frames: Vec<PhysFrame>) -> Self {
        BootInfoFrameAllocator {
            memory_map,
            frames,
            next: 0,
        }
    }
}

impl BootInfoFrameAllocator {
    pub fn into_vec(self) -> Vec<PhysFrame> {
        self.frames
    }
}



impl BootInfoFrameAllocator {
    pub unsafe fn init_temp(memory_map: &'static [MemoryRegion]) -> ([Option<PhysFrame>; 512], &'static [MemoryRegion])

{
        println!("Entered BootInfoFrameAllocator::init_temp");
        println!("BootInfoFrameAllocator::init_temp: memory_map.len = {}", memory_map.len());
//---------------------------------------------------------------------
//Debug code
        for (i, region) in memory_map.iter().enumerate() {
        println!(
        "Region {}: start={:#x}, end={:#x}, kind={:?}",
        i, region.start, region.end, region.kind
    );
}
//end of debug code
//--------------------------------------------------------------------- 


        let mut temp_frames: [Option<PhysFrame>; 512] = [None; 512];
        let mut count = 0;

        for region in memory_map.iter() {
    for addr in (region.start..region.end).step_by(4096) {
        if count >= temp_frames.len() {
            break;
        }

        let frame = PhysFrame::containing_address(PhysAddr::new(addr));
        if frame.start_address().as_u64() < 0x10000 {
            continue;
        }

    //    println!("Adding frame: {:#x}", addr);
        temp_frames[count] = Some(frame);
        count += 1;
    }

    if count >= temp_frames.len() {
        break;
    }
}

    

(temp_frames, memory_map)


        

        
    }
}

   impl BootInfoFrameAllocator {
    /// Full allocator — requires heap to be initialized
    pub unsafe fn init(memory_map: &'static [MemoryRegion]) -> Self {
        println!("Entered BootInfoFrameAllocator::init");
        println!("memory_map.len = {}", memory_map.len());

        let mut frames = Vec::new();

        for region in memory_map.iter() {
            for addr in (region.start..region.end).step_by(4096) {
                let frame = PhysFrame::containing_address(PhysAddr::new(addr));
                if frame.start_address().as_u64() < 0x10000 {
                    continue;
                }

                frames.push(frame);
            }
        }

        BootInfoFrameAllocator {
            memory_map,
            frames,
            next: 0,
        }
    }
}
 



unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
   fn allocate_frame(&mut self) -> Option<PhysFrame> {
    if self.next >= self.frames.len() {
        return None;
    }
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



