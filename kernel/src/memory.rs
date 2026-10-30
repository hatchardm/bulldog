use boot_proto::{MemoryRegion, MemoryRegionKind};

use x86_64::{
    PhysAddr, VirtAddr,
    structures::paging::{
        FrameAllocator, Mapper, OffsetPageTable, Page, PageTable, PageTableFlags,
        PhysFrame, Size4KiB, Translate,
    },
    registers::control::Cr3,
};

use log::{info, debug, warn, error};
use spin::Mutex;

extern crate alloc;
use alloc::vec::Vec;
use alloc::collections::BTreeSet;

use crate::elf::loader::{SegmentFlags, ElfError};
use crate::elf::types::Elf64_Ehdr;
use crate::elf::loader::{validate_elf_header, get_program_headers, load_segments};
use crate::MAPPER;
use crate::FRAME_ALLOCATOR;

// ---------------------------------------------------------------------------
// Track pages mapped for the current user program
// ---------------------------------------------------------------------------

/// Virtual page starts (4KiB) belonging to the current user process.
static USER_PAGES: Mutex<BTreeSet<u64>> = Mutex::new(BTreeSet::new());

fn track_user_page(page: Page<Size4KiB>) {
    USER_PAGES.lock().insert(page.start_address().as_u64());
}

/// Unmap every page recorded for the current user program and free frames.
/// Call from sys_exit before chaining or going idle.
pub fn unmap_current_user() {
    let pages: Vec<u64> = {
        let mut g = USER_PAGES.lock();
        let v: Vec<u64> = g.iter().copied().collect();
        g.clear();
        v
    };

    if pages.is_empty() {
        info!("unmap_current_user: nothing to unmap");
        return;
    }

    info!("unmap_current_user: {} pages", pages.len());

    let mut mapper_guard = MAPPER.lock();
    let mapper = mapper_guard.as_mut().unwrap();

    for va in pages {
        let page = Page::<Size4KiB>::containing_address(VirtAddr::new(va));
        match mapper.unmap(page) {
            Ok((_frame, flush)) => {
                flush.flush();
                // Frame recycle later when BulldogFrameAllocator::free_frame exists
            }
            Err(e) => {
                warn!("unmap_current_user: page {:#x} failed: {:?}", va, e);
            }
        }
    }

    info!("unmap_current_user: done");
}

// ---------------------------------------------------------------------------
// Page table helpers
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Frame allocators
// ---------------------------------------------------------------------------

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


pub struct FrameBitmap {
    bits: *mut [u8; 32768],
    base_address: u64,
    frame_count: usize,
}

static mut BITMAP: [u8; 32768] = [0; 32768];

impl FrameBitmap {
    pub fn new() -> Self {
        unsafe {
            FrameBitmap {
                bits: &raw mut BITMAP,
                base_address: 0x100000,
                frame_count: 262_144,
            }
        }
    }

    fn as_slice(&self) -> &[u8; 32768] {
        unsafe { &*self.bits }
    }

    fn as_mut_slice(&mut self) -> &mut [u8; 32768] {
        unsafe { &mut *self.bits }
    }

    pub fn contains(&self, frame: PhysFrame) -> bool {
        let index = frame.start_address().as_u64() / 4096;
        let byte = (index / 8) as usize;
        let bit = (index % 8) as u8;

        if byte >= self.as_slice().len() {
            error!("Frame {:?} out of bounds for bitmap", frame);
            return false;
        }

        self.as_slice()[byte] & (1 << bit) != 0
    }

    pub fn all_frames(&self) -> impl Iterator<Item = PhysFrame> {
        (0..self.frame_count).map(move |i| {
            let addr = self.base_address + (i as u64) * 4096;
            PhysFrame::containing_address(PhysAddr::new(addr))
        })
    }

    pub fn is_used(&self, frame: PhysFrame) -> bool {
        let index = frame.start_address().as_u64() / 4096;
        let byte = (index / 8) as usize;
        let bit = (index % 8) as u8;
        self.as_slice()[byte] & (1 << bit) != 0
    }

    pub fn iter_used_frames(&self) -> impl Iterator<Item = PhysFrame> + '_ {
        self.as_slice().iter().enumerate().flat_map(|(byte_index, byte)| {
            (0..8).filter_map(move |bit| {
                if byte & (1 << bit) != 0 {
                    let frame_number = byte_index * 8 + bit as usize;
                    Some(PhysFrame::containing_address(PhysAddr::new(
                        (frame_number * 4096) as u64,
                    )))
                } else {
                    None
                }
            })
        })
    }

    pub fn mark_used(&mut self, frame: PhysFrame) -> bool {
        let index = frame.start_address().as_u64() / 4096;
        let byte = (index / 8) as usize;
        let bit = (index % 8) as u8;

        if byte >= self.as_mut_slice().len() {
            error!("Frame {:?} out of bounds for bitmap", frame);
            return false;
        }

        self.as_mut_slice()[byte] |= 1 << bit;
        true
    }

    pub fn mark_free(&mut self, frame: PhysFrame) {
        let index = frame.start_address().as_u64() / 4096;
        let byte = (index / 8) as usize;
        let bit = (index % 8) as u8;

        if byte >= self.as_mut_slice().len() {
            return;
        }

        self.as_mut_slice()[byte] &= !(1 << bit);
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

// ---------------------------------------------------------------------------
// Mapping helpers
// ---------------------------------------------------------------------------

pub fn map_lapic_mmio(
    mapper: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) {
    info!("Mapping LAPIC MMIO region...");

    let virt = VirtAddr::new(crate::apic::LAPIC_VIRT_BASE);
    let phys = PhysAddr::new(0xFEE00000);
    let page = Page::containing_address(virt);
    let frame = PhysFrame::containing_address(phys);
    let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::NO_EXECUTE;

    unsafe {
        mapper
            .map_to(page, frame, flags, frame_allocator)
            .expect("LAPIC map failed")
            .flush();
    }

    info!("LAPIC MMIO fully mapped");
}

pub fn map_page(
    mapper: &mut (impl Mapper<Size4KiB> + Translate),
    virt: VirtAddr,
    phys: PhysAddr,
    flags: PageTableFlags,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) {
    let page = Page::containing_address(virt);
    let frame = PhysFrame::containing_address(phys);

    if mapper.translate_addr(virt).is_some() {
        let _ = unsafe { mapper.update_flags(page, flags) };
        return;
    }

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

pub fn map_elf_segment(
    mapper: &mut (impl Mapper<Size4KiB> + Translate),
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
    vaddr_start: u64,
    mem_size: usize,
    file_bytes: &[u8],
    flags: &SegmentFlags,
) -> Result<(), ElfError> {
    const HHDM: u64 = 0xffff_8000_0000_0000;

    let start = VirtAddr::new(vaddr_start);
    let end = start + (mem_size as u64).saturating_sub(1);
    let start_page = Page::containing_address(start);
    let end_page = Page::containing_address(end);

    let mut page_flags =
        PageTableFlags::PRESENT | PageTableFlags::USER_ACCESSIBLE | PageTableFlags::WRITABLE;
    if !flags.executable {
        page_flags |= PageTableFlags::NO_EXECUTE;
    }

    let parent_flags =
        PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::USER_ACCESSIBLE;

    for page in Page::range_inclusive(start_page, end_page) {
        if mapper.translate_addr(page.start_address()).is_some() {
            // Already mapped (e.g. overlapping segment) — still track for unmap
            track_user_page(page);
            continue;
        }

        let frame = frame_allocator
            .allocate_frame()
            .ok_or(ElfError::BadPhSize)?;

        unsafe {
            mapper
                .map_to_with_table_flags(page, frame, page_flags, parent_flags, frame_allocator)
                .map_err(|_| ElfError::BadPhSize)?
                .flush();
        }

        track_user_page(page);
    }

    // Copy file bytes through HHDM (phys), not the user virtual address
    let mut copied = 0usize;
    let mut va = vaddr_start;
    while copied < file_bytes.len() {
        let page_off = (va & 0xfff) as usize;
        let chunk = core::cmp::min(4096 - page_off, file_bytes.len() - copied);

        let phys = mapper
            .translate_addr(VirtAddr::new(va))
            .ok_or(ElfError::BadPhSize)?
            .as_u64();

        unsafe {
            core::ptr::copy_nonoverlapping(
                file_bytes.as_ptr().add(copied),
                (HHDM + phys) as *mut u8,
                chunk,
            );
        }

        copied += chunk;
        va += chunk as u64;
    }

    // Zero BSS tail via HHDM
    if mem_size > file_bytes.len() {
        let mut zeroed = 0usize;
        let mut va = vaddr_start + file_bytes.len() as u64;
        let total = mem_size - file_bytes.len();

        while zeroed < total {
            let page_off = (va & 0xfff) as usize;
            let chunk = core::cmp::min(4096 - page_off, total - zeroed);

            let phys = mapper
                .translate_addr(VirtAddr::new(va))
                .ok_or(ElfError::BadPhSize)?
                .as_u64();

            unsafe {
                core::ptr::write_bytes((HHDM + phys) as *mut u8, 0, chunk);
            }

            zeroed += chunk;
            va += chunk as u64;
        }
    }

    if !flags.writable {
        let mut ro_flags = PageTableFlags::PRESENT | PageTableFlags::USER_ACCESSIBLE;
        if !flags.executable {
            ro_flags |= PageTableFlags::NO_EXECUTE;
        }
        for page in Page::range_inclusive(start_page, end_page) {
            let _ = unsafe { mapper.update_flags(page, ro_flags) };
        }
    }

    Ok(())
}

pub fn load_elf_into_address_space(
    mapper: &mut (impl Mapper<Size4KiB> + Translate),
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
    elf_data: &[u8],
) -> Result<u64, ElfError> {
    let hdr = unsafe { &*(elf_data.as_ptr() as *const Elf64_Ehdr) };

    validate_elf_header(hdr)?;
    let phdrs = get_program_headers(elf_data, hdr)?;

    load_segments(elf_data, hdr, phdrs, |vaddr, mem_size, file_bytes, flags| {
        map_elf_segment(mapper, frame_allocator, vaddr, mem_size, file_bytes, flags)
            .map_err(|_| ())
    })?;

    Ok(hdr.e_entry)
}

pub fn map_user_segment(
    vaddr: u64,
    mem_size: usize,
    file_bytes: &[u8],
    flags: &crate::elf::loader::SegmentFlags,
) -> Result<(), ()> {
    info!("map_user_segment enter vaddr={:#x}", vaddr);
    let mut mapper_guard = MAPPER.lock();
    let mapper = mapper_guard.as_mut().unwrap();

    let mut fa_guard = FRAME_ALLOCATOR.lock();
    let fa = fa_guard.as_mut().unwrap();

    map_elf_segment(mapper, fa, vaddr, mem_size, file_bytes, flags).map_err(|_| ())
}

pub fn map_user_page(
    mapper: &mut (impl Mapper<Size4KiB> + Translate),
    fa: &mut impl FrameAllocator<Size4KiB>,
    vaddr: u64,
) -> Result<(), ()> {
    let virt = VirtAddr::new(vaddr);
    let page = Page::<Size4KiB>::containing_address(virt);

    let frame = fa.allocate_frame().ok_or(())?;

    let flags =
        PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::USER_ACCESSIBLE;

    map_page(mapper, virt, frame.start_address(), flags, fa);
    track_user_page(page);
    Ok(())
}

pub fn alloc_user_stack() -> Result<u64, ()> {
    let stack_size: usize = 64 * 1024;
    let base: u64 = 0x0000_7FFF_FF00_0000;
    let top = base + stack_size as u64;

    let mut mapper_guard = MAPPER.lock();
    let mapper = mapper_guard.as_mut().unwrap();
    let mut fa_guard = FRAME_ALLOCATOR.lock();
    let fa = fa_guard.as_mut().unwrap();

    for offset in (0..stack_size).step_by(4096) {
        map_user_page(mapper, fa, base + offset as u64)?;
    }

    Ok(top)
}

/// Set USER_ACCESSIBLE (and clear NX) on every level of the page walk for `vaddr`.
pub unsafe fn force_user_access(vaddr: VirtAddr, phys_offset: u64) {
    use x86_64::structures::paging::page_table::PageTableFlags as Ptf;

    let (cr3_frame, _) = Cr3::read();
    let mut table_phys = cr3_frame.start_address().as_u64();
    let va = vaddr.as_u64();

    let i4 = ((va >> 39) & 0x1ff) as usize;
    let i3 = ((va >> 30) & 0x1ff) as usize;
    let i2 = ((va >> 21) & 0x1ff) as usize;
    let i1 = ((va >> 12) & 0x1ff) as usize;

    let userish = Ptf::PRESENT | Ptf::WRITABLE | Ptf::USER_ACCESSIBLE;

    let pml4 = &mut *((phys_offset + table_phys) as *mut PageTable);
    let e4 = &mut pml4[i4];
    if e4.is_unused() {
        log::error!("force_user_access: PML4 empty");
        return;
    }
    {
        let mut f = e4.flags() | userish;
        f.remove(Ptf::NO_EXECUTE);
        e4.set_flags(f);
    }
    table_phys = e4.addr().as_u64();

    let pdpt = &mut *((phys_offset + table_phys) as *mut PageTable);
    let e3 = &mut pdpt[i3];
    if e3.is_unused() {
        log::error!("force_user_access: PDPT empty");
        return;
    }
    if e3.flags().contains(Ptf::HUGE_PAGE) {
        let mut f = e3.flags() | userish;
        f.remove(Ptf::NO_EXECUTE);
        e3.set_flags(f);
        log::info!("force_user_access: USER on PML4 + 1GiB PDPT");
        core::arch::asm!("invlpg [{}]", in(reg) va, options(nostack, preserves_flags));
        return;
    }
    {
        let mut f = e3.flags() | userish;
        f.remove(Ptf::NO_EXECUTE);
        e3.set_flags(f);
    }
    table_phys = e3.addr().as_u64();

    let pd = &mut *((phys_offset + table_phys) as *mut PageTable);
    let e2 = &mut pd[i2];
    if e2.is_unused() {
        log::error!("force_user_access: PD empty");
        return;
    }
    if e2.flags().contains(Ptf::HUGE_PAGE) {
        let mut f = e2.flags() | userish;
        f.remove(Ptf::NO_EXECUTE);
        e2.set_flags(f);
        log::info!("force_user_access: USER on PML4+PDPT+2MiB");
        core::arch::asm!("invlpg [{}]", in(reg) va, options(nostack, preserves_flags));
        return;
    }
    {
        let mut f = e2.flags() | userish;
        f.remove(Ptf::NO_EXECUTE);
        e2.set_flags(f);
    }
    table_phys = e2.addr().as_u64();

    let pt = &mut *((phys_offset + table_phys) as *mut PageTable);
    let e1 = &mut pt[i1];
    if e1.is_unused() {
        log::error!("force_user_access: PT empty");
        return;
    }
    {
        let mut f = e1.flags() | userish;
        f.remove(Ptf::NO_EXECUTE);
        e1.set_flags(f);
    }
    log::info!("force_user_access: USER on full 4KiB chain");
    core::arch::asm!("invlpg [{}]", in(reg) va, options(nostack, preserves_flags));
}