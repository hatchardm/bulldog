#![no_std]

use crate::serial::{serial_print, serial_print_hex_u64, serial_println};
use boot_proto::{BootInfo, MemoryRegionKind};
use x86_64::structures::paging::mapper::MapToError;
use x86_64::{
    PhysAddr, VirtAddr,
    registers::control::{Cr3, Cr3Flags},
    structures::paging::{
        FrameAllocator, Mapper, OffsetPageTable, Page, PageSize, PageTable, PageTableFlags,
        PhysFrame, Size4KiB,
    },
};

use core::cell::OnceCell;
use spin::Mutex;

pub static MAPPER: Mutex<Option<OffsetPageTable<'static>>> = Mutex::new(None);
pub static FRAME_ALLOCATOR: Mutex<Option<BulldogFrameAllocator>> = Mutex::new(None);








fn new_offset_page_table<'a>(
    phys_offset_va: VirtAddr,
) -> OffsetPageTable<'a> {
    // Get the currently active PML4 frame from CR3
    let (pml4_frame, _) = Cr3::read();

    // Convert its physical address to a virtual address via HHDM
    let pml4_virt = phys_offset_va + pml4_frame.start_address().as_u64();

    // Get a mutable reference to the existing PML4
    let pml4_table: &mut PageTable = unsafe { &mut *(pml4_virt.as_mut_ptr()) };

    // Build an OffsetPageTable over the active PML4
    unsafe { OffsetPageTable::new(pml4_table, phys_offset_va) }
}



pub struct BulldogFrameAllocator {
    next: PhysAddr,
    end: PhysAddr,
}

impl BulldogFrameAllocator {


    pub fn new(boot_info: &BootInfo) -> Self {
    let fb_start = boot_info.framebuffer.addr;
    let fb_size  = (boot_info.framebuffer.width as u64)
                 * (boot_info.framebuffer.height as u64) * 4;
    let fb_end   = fb_start + fb_size;

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


    BulldogFrameAllocator { next: start, end }
}



}

unsafe impl FrameAllocator<Size4KiB> for BulldogFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
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
    let mapper = new_offset_page_table(phys_offset_va);

    // store globals
    FRAME_ALLOCATOR.lock().replace(fa);
    MAPPER.lock().replace(mapper);


    // use mutable refs from the globals
    let mut fa_guard = FRAME_ALLOCATOR.lock();
    let fa = fa_guard.as_mut().expect("FRAME_ALLOCATOR not set");

    let mut mapper_guard = MAPPER.lock();
    let mapper = mapper_guard.as_mut().expect("MAPPER not set");


    const BOOT_IDENTITY_LIMIT: u64 = 512 * 1024 * 1024;

    for i in 0..boot_info.memory_region_count {
        let region = &boot_info.memory_regions_buffer[i];
        if region.kind != MemoryRegionKind::Usable {
            continue;
        }

        let mut start = PhysAddr::new(region.start);
        let end = PhysAddr::new(region.end);

        while start < end {
            if start.as_u64() < BOOT_IDENTITY_LIMIT {
                start += Size4KiB::SIZE;
                continue;
            }

            let frame = PhysFrame::<Size4KiB>::containing_address(start);
            let virt  = phys_offset_va + frame.start_address().as_u64();
            let page  = Page::<Size4KiB>::containing_address(virt);

            let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;

            mapper
                .map_to(page, frame, flags, fa)
                .expect("map_to HHDM")
                .flush();

            start += Size4KiB::SIZE;
        }
    }

    // framebuffer mapping: same change, use `fa` not `&mut fa`
    if boot_info.framebuffer_present != 0 {
        let fb = &boot_info.framebuffer;
        let fb_start = PhysAddr::new(fb.addr);
        let fb_size  = (fb.stride as u64) * (fb.height as u64) * 4;
        let fb_end   = fb_start + fb_size;

        let mut cur = fb_start;
        while cur < fb_end {
            let frame = PhysFrame::<Size4KiB>::containing_address(cur);
            let virt  = phys_offset_va + frame.start_address().as_u64();
            let page  = Page::<Size4KiB>::containing_address(virt);

            let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;

            mapper
                .map_to(page, frame, flags, fa)
                .expect("map_to framebuffer")
                .flush();

            cur += Size4KiB::SIZE;
        }

        boot_info.framebuffer_virt = (phys_offset_va + fb.addr).as_u64();
    }
}


