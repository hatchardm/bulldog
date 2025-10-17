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
use alloc::collections::BTreeSet;

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
    pub allocated: FrameBitmap,
}

pub struct FrameBitmap {
    bits: *mut [u8; 32768],
    base_address: u64,     // e.g., 0x100000
    frame_count: usize,    // e.g., 262_144
}

static mut BITMAP: [u8; 32768] = [0; 32768];

impl FrameBitmap {
    pub fn new() -> Self {
        unsafe {
            FrameBitmap {
                bits: &raw mut BITMAP,
                base_address: 0x100000, // Start at 1 MiB
                frame_count: 262_144,   // 1 GiB of 4 KiB frames
            }
        }
    }
}

impl FrameBitmap {
    fn as_slice(&self) -> &[u8; 32768] {
        unsafe { &*self.bits }
    }

  pub fn contains(&self, frame: PhysFrame) -> bool {
    let index = frame.start_address().as_u64() / 4096;
    let byte = (index / 8) as usize;
    let bit = (index % 8) as u8;

    if byte >= self.as_slice().len() {
        println!("Frame {:?} out of bounds for bitmap", frame);
        return false;
    }

    self.as_slice()[byte] & (1 << bit) != 0
}


}

impl FrameBitmap {
    pub fn all_frames(&self) -> impl Iterator<Item = PhysFrame> {
        (0..self.frame_count).map(move |i| {
            let addr = self.base_address + (i as u64) * 4096;
            PhysFrame::containing_address(PhysAddr::new(addr))
        })
    }
}



impl FrameBitmap {
   
  pub fn is_used(&self, frame: PhysFrame) -> bool {
    let index = frame.start_address().as_u64() / 4096;
    let byte = (index / 8) as usize;
    let bit = (index % 8) as u8;
    self.as_slice()[byte] & (1 << bit) != 0
}

}



impl FrameBitmap {
   
    pub fn iter_used_frames(&self) -> impl Iterator<Item = PhysFrame> + '_ {
        self.as_slice().iter().enumerate().flat_map(|(byte_index, byte)| {
            (0..8).filter_map(move |bit| {
                if byte & (1 << bit) != 0 {
                    let frame_number = byte_index * 8 + bit as usize;
                    Some(PhysFrame::containing_address(PhysAddr::new((frame_number * 4096) as u64)))
                } else {
                    None
                }
            })
        })
    }
}


impl FrameBitmap {

fn as_mut_slice(&mut self) -> &mut [u8; 32768] {
    unsafe { &mut *self.bits }
}


  pub fn mark_used(&mut self, frame: PhysFrame) -> bool {
    let index = frame.start_address().as_u64() / 4096;
    let byte = (index / 8) as usize;
    let bit = (index % 8) as u8;

    if byte >= self.as_mut_slice().len() {
        println!("Frame {:?} out of bounds for bitmap", frame);
        return false;
    }

    self.as_mut_slice()[byte] |= 1 << bit;
    true
}


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
println!("Entered BootInfoFrameAllocator::new");

        BootInfoFrameAllocator {
            memory_map,
            frames,
            next: 0,
            allocated: FrameBitmap::new(),

        }
    }
}

impl BootInfoFrameAllocator {
   pub fn allocated_frames(&self) -> impl Iterator<Item = PhysFrame> {
    self.allocated.iter_used_frames()
   }
}


impl BootInfoFrameAllocator {
    pub fn into_vec(self) -> Vec<PhysFrame> {
        self.frames
    }
}

impl BootInfoFrameAllocator {
    pub fn is_allocated(&self, frame: PhysFrame) -> bool {
    self.allocated.contains(frame)
}

}


impl BootInfoFrameAllocator {
    pub unsafe fn init_temp(memory_map: &'static [MemoryRegion]) -> ([Option<PhysFrame>; 512], &'static [MemoryRegion])

{
        println!("Entered BootInfoFrameAllocator::init_temp");
        println!("BootInfoFrameAllocator::init_temp: memory_map.len = {}", memory_map.len());
//---------------------------------------------------------------------
//Debug code
     //   for (i, region) in memory_map.iter().enumerate() {
      //  println!(
     //   "Region {}: start={:#x}, end={:#x}, kind={:?}",
     //   i, region.start, region.end, region.kind
  //  );
//}
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
    pub fn mark_used_frames(&mut self) {
        println!("Starting mark_used_frames()");

        for region in self.memory_map.iter() {
            if region.start >= region.end {
                println!("Skipping invalid region: start={:#x}, end={:#x}", region.start, region.end);
                continue;
            }

            match region.kind {
                MemoryRegionKind::Usable => {
                    // Optional: log skipped usable regions
                    // println!("Skipping usable region: {:#x} - {:#x}", region.start, region.end);
                    continue;
                }
                _ => {
                    println!(
                        "Marking used region: start={:#x}, end={:#x}, kind={:?}",
                        region.start, region.end, region.kind
                    );

                    for addr in (region.start..region.end).step_by(4096) {
                        let frame = PhysFrame::containing_address(PhysAddr::new(addr));
                        self.allocated.mark_used(frame);
                    }
                }
            }
        }
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
            allocated: FrameBitmap::new(),

        }
    }
}
 



unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
   fn allocate_frame(&mut self) -> Option<PhysFrame> {
    if self.next >= self.frames.len() {
        return None;
    }
    let frame = self.frames[self.next];
    self.next += 1;
    self.allocated.mark_used(frame); // ✅ track it

    Some(frame)
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

pub fn find_unused_frame(allocator: &FrameBitmap) -> Option<PhysFrame> {
    for frame in allocator.all_frames() {
        if !allocator.is_used(frame) {
            return Some(frame);
        }
    }
    None
}




