#![no_std]

extern crate alloc;
use alloc::vec::Vec;

use crate::serial::{serial_print, serial_print_hex_u64, serial_println};
use boot_proto::{BootInfo, MemoryRegionKind};
use x86_64::{
    PhysAddr, VirtAddr,
    registers::control::Cr3,
    structures::paging::{
        FrameAllocator, OffsetPageTable, PageSize, PageTable, PhysFrame, Size4KiB,
    },
};
use spin::Mutex;

pub static MAPPER: Mutex<Option<OffsetPageTable<'static>>> = Mutex::new(None);
pub static FRAME_ALLOCATOR: Mutex<Option<BulldogFrameAllocator>> = Mutex::new(None);

fn new_offset_page_table<'a>(phys_offset_va: VirtAddr) -> OffsetPageTable<'a> {
    let (pml4_frame, _) = Cr3::read();
    let pml4_virt = phys_offset_va + pml4_frame.start_address().as_u64();
    let pml4_table: &mut PageTable = unsafe { &mut *(pml4_virt.as_mut_ptr()) };
    unsafe { OffsetPageTable::new(pml4_table, phys_offset_va) }
}

pub struct BulldogFrameAllocator {
    next: PhysAddr,
    end: PhysAddr,
    free_list: Vec<PhysFrame<Size4KiB>>,
}

impl BulldogFrameAllocator {
    pub fn new(boot_info: &BootInfo) -> Self {
        let fb_start = boot_info.framebuffer.addr;
        let fb_size = (boot_info.framebuffer.width as u64)
            * (boot_info.framebuffer.height as u64)
            * 4;
        let fb_end = fb_start + fb_size;

        let mut start = PhysAddr::new(0);
        let mut end = PhysAddr::new(0);

        for i in 0..boot_info.memory_region_count {
            let region = &boot_info.memory_regions_buffer[i];
            if region.kind != MemoryRegionKind::Usable {
                continue;
            }

            // skip regions that overlap the framebuffer
            if !(fb_end <= region.start || fb_start >= region.end) {
                continue;
            }

            if region.end - region.start < 0x200000 {
                continue;
            }

            start = PhysAddr::new(region.start + 0x200000);
            end = PhysAddr::new(region.end);
            break;
        }

        serial_println("=== FRAME ALLOCATOR CHOICE ===");
        serial_print("start = ");
        serial_print_hex_u64(start.as_u64());
        serial_print(", end = ");
        serial_print_hex_u64(end.as_u64());
        serial_println("");

        BulldogFrameAllocator {
            next: start,
            end,
            free_list: Vec::new(),
        }
    }

    /// Return a frame to the free pool (from unmap_current_user).
    pub fn free_frame(&mut self, frame: PhysFrame<Size4KiB>) {
        self.free_list.push(frame);
    }

    pub fn free_list_len(&self) -> usize {
        self.free_list.len()
    }
}

unsafe impl FrameAllocator<Size4KiB> for BulldogFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        if let Some(frame) = self.free_list.pop() {
            return Some(frame);
        }

        if self.next + Size4KiB::SIZE > self.end {
            return None;
        }

        let frame = PhysFrame::<Size4KiB>::containing_address(self.next);
        self.next += Size4KiB::SIZE;
        Some(frame)
    }
}

pub unsafe fn init(boot_info: &mut BootInfo) {
    let phys_offset = boot_info.physical_memory_offset;
    let phys_offset_va = VirtAddr::new(phys_offset);

    let fa = BulldogFrameAllocator::new(boot_info);
    FRAME_ALLOCATOR.lock().replace(fa);

    let mapper = new_offset_page_table(phys_offset_va);
    MAPPER.lock().replace(mapper);

    if boot_info.framebuffer_present != 0 {
        boot_info.framebuffer_virt = (phys_offset_va + boot_info.framebuffer.addr).as_u64();
    }

    serial_println("paging::init finished (using bootloader maps)");
}
