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


fn new_offset_page_table<'a>(
    phys_offset_va: VirtAddr,
    fa: &mut BulldogFrameAllocator,
) -> (OffsetPageTable<'a>, PhysFrame<Size4KiB>) {
    let pml4_frame = fa.allocate_frame().expect("no frame for PML4");
    let pml4_virt = phys_offset_va + pml4_frame.start_address().as_u64();

    let pml4_table: &mut PageTable = unsafe { &mut *(pml4_virt.as_mut_ptr()) };
    pml4_table.zero();

    let mapper = unsafe { OffsetPageTable::new(pml4_table, phys_offset_va) };
    (mapper, pml4_frame)
}

pub struct BulldogFrameAllocator {
    next: PhysAddr,
    end: PhysAddr,
}

impl BulldogFrameAllocator {
    pub fn new(boot_info: &BootInfo) -> Self {
        let mut start = PhysAddr::new(0);
        let mut end = PhysAddr::new(0);

        for i in 0..boot_info.memory_region_count {
            let region = &boot_info.memory_regions_buffer[i];
            if region.kind != MemoryRegionKind::Usable {
                continue;
            }

            if region.end - region.start < 0x200000 {
                continue;
            }

            start = PhysAddr::new(region.start + 0x200000); // skip first 2MiB
            end = PhysAddr::new(region.end);
            break;
        }

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

    // 1) Build frame allocator from memory_regions_buffer
    let mut fa = BulldogFrameAllocator::new(boot_info);

    // 2) Create fresh PML4
    let (mut mapper, pml4_frame) = new_offset_page_table(phys_offset_va, &mut fa);

    // 2.5) Identity-map kernel physical region
    let k_start = PhysAddr::new(boot_info.kernel_phys_start);
    let k_end = PhysAddr::new(boot_info.kernel_phys_end);

    let mut cur = k_start;
    while cur < k_end {
        let frame = PhysFrame::<Size4KiB>::containing_address(cur);
        let page = Page::<Size4KiB>::containing_address(VirtAddr::new(cur.as_u64()));

        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;

        unsafe {
            mapper
                .map_to(page, frame, flags, &mut fa)
                .expect("map_to kernel identity")
                .flush();
        }

        cur += Size4KiB::SIZE;
    }

for i in 0..boot_info.memory_region_count {
    let region = &boot_info.memory_regions_buffer[i];
    if region.kind != MemoryRegionKind::Usable {
        continue;
    }

    let mut start = PhysAddr::new(region.start);
    let end = PhysAddr::new(region.end);

    while start < end {
        let frame = PhysFrame::<Size4KiB>::containing_address(start);
        let virt  = phys_offset_va + frame.start_address().as_u64();
        let page  = Page::<Size4KiB>::containing_address(virt);

        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;

        unsafe {
            mapper
                .map_to(page, frame, flags, &mut fa)
                .expect("map_to HHDM")
                .flush();
        }

        start += Size4KiB::SIZE;
    }
}

if boot_info.framebuffer_present != 0 {
    let fb = &boot_info.framebuffer;
    let fb_start = PhysAddr::new(fb.addr);
    let fb_size  = (fb.width as u64) * (fb.height as u64) * 4;
    let fb_end   = fb_start + fb_size;

    let mut cur = fb_start;
    while cur < fb_end {
        let frame = PhysFrame::<Size4KiB>::containing_address(cur);
        let virt  = phys_offset_va + frame.start_address().as_u64();
        let page  = Page::<Size4KiB>::containing_address(virt);

        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;

        unsafe {
            mapper
                .map_to(page, frame, flags, &mut fa)
                .expect("map_to framebuffer")
                .flush();
        }

        cur += Size4KiB::SIZE;
    }

    // ⭐ Publish the mapped virtual base address
    boot_info.framebuffer_virt = (phys_offset_va + fb.addr).as_u64();
}





    
}
