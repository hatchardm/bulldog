//! Minimal paging: identity + HHDM using 2 MiB pages for the first 4 GiB.
//! Covers QEMU RAM, FAT image, and GOP framebuffer at phys 0x8000_0000.

use uefi::mem::memory_map::MemoryMapOwned;
use x86_64::{
    PhysAddr, VirtAddr,
    registers::control::Cr3,
    structures::paging::{
        Page, PageSize, PageTable, PageTableFlags, PhysFrame, Size2MiB,
    },
};

pub const HHDM_BASE: u64 = 0xffff_8000_0000_0000;

/// Phys 0 .. 4 GiB (includes FB at 0x8000_0000).
const MAP_SIZE: u64 = 4 * 1024 * 1024 * 1024;
/// One PD covers 1 GiB of 2 MiB pages.
const GIBS: usize = 4;

#[repr(align(4096))]
struct AlignedPageTable(PageTable);

static mut PML4: AlignedPageTable = AlignedPageTable(PageTable::new());
static mut PDPT_ID: AlignedPageTable = AlignedPageTable(PageTable::new());
static mut PDPT_HHDM: AlignedPageTable = AlignedPageTable(PageTable::new());

static mut PD_ID: [AlignedPageTable; GIBS] = [
    AlignedPageTable(PageTable::new()),
    AlignedPageTable(PageTable::new()),
    AlignedPageTable(PageTable::new()),
    AlignedPageTable(PageTable::new()),
];

static mut PD_HHDM: [AlignedPageTable; GIBS] = [
    AlignedPageTable(PageTable::new()),
    AlignedPageTable(PageTable::new()),
    AlignedPageTable(PageTable::new()),
    AlignedPageTable(PageTable::new()),
];

pub fn setup_minimal_paging(_memory_map: &MemoryMapOwned) {
    unsafe {
        let pml4_phys = PhysAddr::new(&raw const PML4 as *const _ as u64);
        let pdpt_id_phys = PhysAddr::new(&raw const PDPT_ID as *const _ as u64);
        let pdpt_hhdm_phys = PhysAddr::new(&raw const PDPT_HHDM as *const _ as u64);

        let pml4 = &mut *(&raw mut PML4.0);
        let pdpt_id = &mut *(&raw mut PDPT_ID.0);
        let pdpt_hhdm = &mut *(&raw mut PDPT_HHDM.0);

        pml4.zero();
        pdpt_id.zero();
        pdpt_hhdm.zero();

        // PML4[0] → identity PDPT
        pml4[0].set_addr(
            pdpt_id_phys,
            PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
        );

        // PML4[256] → HHDM PDPT (0xffff_8000_0000_0000)
        pml4[256].set_addr(
            pdpt_hhdm_phys,
            PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
        );

        // PDPT[n] → PD[n] for each GiB
        for n in 0..GIBS {
            let pd_id_phys = PhysAddr::new(&raw const PD_ID[n] as *const _ as u64);
            let pd_hhdm_phys = PhysAddr::new(&raw const PD_HHDM[n] as *const _ as u64);

            (&mut *(&raw mut PD_ID[n].0)).zero();
            (&mut *(&raw mut PD_HHDM[n].0)).zero();

            pdpt_id[n].set_addr(
                pd_id_phys,
                PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
            );
            pdpt_hhdm[n].set_addr(
                pd_hhdm_phys,
                PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
            );
        }

        let flags = PageTableFlags::PRESENT
            | PageTableFlags::WRITABLE
            | PageTableFlags::HUGE_PAGE;

        let mut phys = 0u64;
        while phys < MAP_SIZE {
            let frame = PhysFrame::<Size2MiB>::containing_address(PhysAddr::new(phys));
            let gib = (phys / (1024 * 1024 * 1024)) as usize; // which PD

            // Identity
            let page_id = Page::<Size2MiB>::containing_address(VirtAddr::new(phys));
            let idx: usize = page_id.p2_index().into();
            (&mut *(&raw mut PD_ID[gib].0))[idx].set_addr(frame.start_address(), flags);

            // HHDM
            let page_hhdm =
                Page::<Size2MiB>::containing_address(VirtAddr::new(HHDM_BASE + phys));
            let idx_h: usize = page_hhdm.p2_index().into();
            (&mut *(&raw mut PD_HHDM[gib].0))[idx_h].set_addr(frame.start_address(), flags);

            phys += Size2MiB::SIZE;
        }

        let pml4_frame = PhysFrame::containing_address(pml4_phys);
        let (_, cr3_flags) = Cr3::read();
        Cr3::write(pml4_frame, cr3_flags);
    }
}