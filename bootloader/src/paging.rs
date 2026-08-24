// bootloader/src/paging.rs
use uefi::mem::memory_map::MemoryMapOwned;
use uefi::boot::MemoryType;
use uefi::mem::memory_map::MemoryMap;
use x86_64::{
    PhysAddr, VirtAddr,
    structures::paging::{
        PageTable, PageTableFlags,
        PhysFrame, Size2MiB, PageSize, Page,
    },
    registers::control::Cr3,
};



pub const HHDM_BASE: u64 = 0xffff800000000000;

fn find_conventional_region(memory_map: &MemoryMapOwned) -> Option<(u64, u64)> {
    for desc in memory_map.entries() {
        if desc.ty == MemoryType::CONVENTIONAL {
            let start = desc.phys_start;
            let len   = desc.page_count * 4096;
            return Some((start, start + len));
        }
    }
    None
}

#[repr(align(4096))]
struct AlignedPageTable(PageTable);

static mut PML4: AlignedPageTable = AlignedPageTable(PageTable::new());
static mut PDPT: AlignedPageTable = AlignedPageTable(PageTable::new());
static mut PD:   AlignedPageTable = AlignedPageTable(PageTable::new());

pub fn setup_minimal_paging(memory_map: &MemoryMapOwned) {
    // Still assert there is at least one CONVENTIONAL region,
    // but we don't use its start as the mapping base anymore.
    let (_phys_start, _phys_end) =
        find_conventional_region(memory_map).expect("no CONVENTIONAL region");

    // Map 0..512 MiB
    let phys_start = 0u64;
    let phys_end   = 512 * 1024 * 1024;

    unsafe {
        let pml4_phys = PhysAddr::new(&PML4 as *const _ as u64);
        let pdpt_phys = PhysAddr::new(&PDPT as *const _ as u64);
        let pd_phys   = PhysAddr::new(&PD   as *const _ as u64);

        let pml4 = &mut PML4.0;
        let pdpt = &mut PDPT.0;
        let pd   = &mut PD.0;

        pml4.zero();
        pdpt.zero();
        pd.zero();

        // PML4[0] -> PDPT (identity)
        pml4[0].set_addr(pdpt_phys, PageTableFlags::PRESENT | PageTableFlags::WRITABLE);

        // PML4[256] -> PDPT (HHDM at 0xffff8000_0000_0000)
        pml4[256].set_addr(pdpt_phys, PageTableFlags::PRESENT | PageTableFlags::WRITABLE);

        // PDPT[0] -> PD
        pdpt[0].set_addr(pd_phys, PageTableFlags::PRESENT | PageTableFlags::WRITABLE);

        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;

        let mut phys = PhysAddr::new(phys_start);
        while phys < PhysAddr::new(phys_end) {
            let frame = PhysFrame::<Size2MiB>::containing_address(phys);

            // identity
            let page_id = Page::<Size2MiB>::containing_address(
                VirtAddr::new(phys.as_u64()),
            );
            let idx: usize = page_id.p2_index().into();
            pd[idx].set_addr(frame.start_address(), flags | PageTableFlags::HUGE_PAGE);

            // hhdm
            let page_hhdm = Page::<Size2MiB>::containing_address(
                VirtAddr::new(HHDM_BASE + phys.as_u64()),
            );
            let idx_h: usize = page_hhdm.p2_index().into();
            pd[idx_h].set_addr(frame.start_address(), flags | PageTableFlags::HUGE_PAGE);

            phys += Size2MiB::SIZE;
        }

        // Switch to our new PML4
        let pml4_frame = PhysFrame::containing_address(pml4_phys);
        Cr3::write(pml4_frame, Cr3::read().1);
    }
}
