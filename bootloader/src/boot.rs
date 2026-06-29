// boot.rs
use crate::console::Console;
use uefi::system;
use uefi::table::cfg::ConfigTableEntry;
use uefi::boot::{self, MemoryType};
use uefi::mem::memory_map::{MemoryMap, MemoryMapOwned};
use log::info;
use core::slice;

use boot_proto::{
    BootInfo as BulldogBootInfo,
    MemoryRegion as BulldogMemoryRegion,
    MemoryRegionKind as BulldogMemoryRegionKind,
};

use x86_64::structures::paging::Mapper;

use alloc::vec::Vec;

use x86_64::{
    PhysAddr, VirtAddr,
    structures::paging::{
        PageTable, PageTableFlags, PhysFrame, Size4KiB,
        OffsetPageTable, FrameAllocator,
    },
    registers::control::Cr3,
};


extern crate alloc;


#[repr(align(16))]
struct AlignedKernelStack([u8; 100 * 1024]);

#[unsafe(no_mangle)]
static mut KERNEL_STACK: AlignedKernelStack = AlignedKernelStack([0; 100 * 1024]);

// ============================================================
//  Top‑level ELF definitions
// ============================================================

#[repr(C)]
struct Elf64Ehdr {
    e_ident: [u8; 16],
    e_type: u16,
    e_machine: u16,
    e_version: u32,
    e_entry: u64,
    e_phoff: u64,
    e_shoff: u64,
    e_flags: u32,
    e_ehsize: u16,
    e_phentsize: u16,
    e_phnum: u16,
    e_shentsize: u16,
    e_shnum: u16,
    e_shstrndx: u16,
}

#[repr(C)]
struct Elf64Phdr {
    p_type: u32,
    p_flags: u32,
    p_offset: u64,
    p_vaddr: u64,
    p_paddr: u64,
    p_filesz: u64,
    p_memsz: u64,
    p_align: u64,
}

const PT_LOAD: u32 = 1;

// ============================================================
//  Relocation structures
// ============================================================

#[repr(C)]
struct Elf64Shdr {
    sh_name: u32,
    sh_type: u32,
    sh_flags: u64,
    sh_addr: u64,
    sh_offset: u64,
    sh_size: u64,
    sh_link: u32,
    sh_info: u32,
    sh_addralign: u64,
    sh_entsize: u64,
}

#[repr(C)]
struct Elf64Rela {
    r_offset: u64,
    r_info: u64,
    r_addend: i64,
}

const SHT_RELA: u32 = 4;
const R_X86_64_RELATIVE: u32 = 8;

// ============================================================
//  Static memory region buffer
// ============================================================

static mut MEMORY_REGIONS: [core::mem::MaybeUninit<BulldogMemoryRegion>; 256] =
    [const { core::mem::MaybeUninit::uninit() }; 256];

// ============================================================
//  Kernel loader
// ============================================================

pub fn load_kernel(console: &mut Console) -> Result<(*mut u8, usize), ()> {
    use uefi::proto::media::file::{
        File, FileMode, FileAttribute, FileType, FileInfo,
    };
    use uefi::CString16;

    console.write_str("Opening kernel file...\n");

    let image = boot::image_handle();
    let mut fs = boot::get_image_file_system(image)
        .expect("get_image_file_system failed");

    let mut root = fs.open_volume()
        .expect("open_volume failed");

    let kernel_path = CString16::try_from("\\EFI\\BOOT\\kernel.elf")
        .expect("invalid kernel path");

    let kernel_handle = root
        .open(&kernel_path, FileMode::Read, FileAttribute::empty())
        .expect("failed to open kernel");

    let mut kernel_file = match kernel_handle.into_type().expect("into_type failed") {
        FileType::Regular(f) => f,
        _ => {
            console.write_str("Kernel is not a regular file!\n");
            return Err(());
        }
    };

    let mut info_buf = [0u8; 512];
    let info = kernel_file
        .get_info::<FileInfo>(&mut info_buf)
        .expect("get_info failed");

    let file_size = info.file_size() as usize;

    let buf_nn = boot::allocate_pool(MemoryType::LOADER_DATA, file_size)
        .expect("allocate_pool failed");
    let buf_ptr = buf_nn.as_ptr();

    let buf_slice = unsafe { slice::from_raw_parts_mut(buf_ptr, file_size) };

    kernel_file.read(buf_slice).expect("kernel read failed");

    console.write_str("Kernel loaded into memory.\n");

    Ok((buf_ptr, file_size))
}

// ============================================================
//  Memory map helpers (for BootInfo)
// ============================================================



/// Fill BootInfo.memory_regions / memory_region_count from an existing MemoryMapOwned.
pub fn fill_memory_regions_from_map(
    boot_info: &mut BulldogBootInfo,
    memory_map: &MemoryMapOwned,
) {
    let mut idx = 0usize;

    for desc in memory_map.entries() {
        if idx >= unsafe { MEMORY_REGIONS.len() } {
            break;
        }

        let start = desc.phys_start;
        let len_bytes = desc.page_count * 4096;
        let end = start + len_bytes;

        let kind = match desc.ty {
            MemoryType::CONVENTIONAL => BulldogMemoryRegionKind::Usable,
            MemoryType::ACPI_RECLAIM => BulldogMemoryRegionKind::Acpi,
            MemoryType::MMIO | MemoryType::MMIO_PORT_SPACE => BulldogMemoryRegionKind::Mmio,
            _ => BulldogMemoryRegionKind::Reserved,
        };

        let region = BulldogMemoryRegion { start, end, kind };

        unsafe {
            MEMORY_REGIONS[idx].as_mut_ptr().write(region);
        }
        idx += 1;
    }

    let slice = unsafe {
        let ptr = MEMORY_REGIONS.as_ptr() as *const BulldogMemoryRegion;
        core::slice::from_raw_parts(ptr, idx)
    };

    boot_info.memory_regions = slice.as_ptr();
    boot_info.memory_region_count = idx;
}

// ============================================================
//  Relocation pass
// ============================================================

fn apply_relocations(elf: &[u8]) {
    let ehdr = unsafe { &*(elf.as_ptr() as *const Elf64Ehdr) };
    let sh_base = unsafe {
        elf.as_ptr().add(ehdr.e_shoff as usize) as *const Elf64Shdr
    };

    for i in 0..ehdr.e_shnum {
        let sh = unsafe { &*sh_base.add(i as usize) };
        if sh.sh_type != SHT_RELA {
            continue;
        }

        let count = (sh.sh_size / sh.sh_entsize) as usize;

        let rela_slice = unsafe {
            core::slice::from_raw_parts(
                elf.as_ptr().add(sh.sh_offset as usize) as *const Elf64Rela,
                count,
            )
        };

        for rela in rela_slice {
            let r_type = (rela.r_info & 0xff) as u32;
            if r_type != R_X86_64_RELATIVE {
                continue;
            }

            let loc = rela.r_offset as *mut u64;

            unsafe {
                core::ptr::write(loc, rela.r_addend as u64);
            }
        }
    }
}

// ============================================================
//  Jump to kernel
// ============================================================

#[inline(never)]
pub fn jump_to_kernel(
    kernel_ptr: *mut u8,
    kernel_len: usize,
    boot_info: &mut BulldogBootInfo,
) -> ! {
    use core::{mem, ptr};
    use core::slice;

    info!(
        "jump_to_kernel entered: ptr = {:#x}, len = {}",
        kernel_ptr as u64,
        kernel_len
    );

    let elf = unsafe { slice::from_raw_parts(kernel_ptr as *const u8, kernel_len) };

    let ehdr = unsafe { &*(elf.as_ptr() as *const Elf64Ehdr) };
    let ph_base = unsafe { elf.as_ptr().add(ehdr.e_phoff as usize) as *const Elf64Phdr };

    let entry_addr = ehdr.e_entry;

    info!(
        "ELF header: entry = {:#x}, phoff = {:#x}, phnum = {}",
        entry_addr,
        ehdr.e_phoff,
        ehdr.e_phnum
    );

    apply_relocations(elf);

    for i in 0..ehdr.e_phnum {
        let ph_ptr = unsafe { ph_base.add(i as usize) };
        info!("PHDR {}: ph_ptr = {:#x}", i, ph_ptr as u64);

        let ph = unsafe { &*ph_ptr };

        if ph.p_type != PT_LOAD {
            continue;
        }

        info!(
            "PT_LOAD: vaddr = {:#x}, paddr = {:#x}, filesz = {}, memsz = {}",
            ph.p_vaddr,
            ph.p_paddr,
            ph.p_filesz,
            ph.p_memsz,
        );

        let dest = ph.p_vaddr as *mut u8;
        let src = unsafe { elf.as_ptr().add(ph.p_offset as usize) };
        let file_sz = ph.p_filesz as usize;
        let mem_sz = ph.p_memsz as usize;

        info!(
            "PT_LOAD copy: dest = {:#x}, src = {:#x}, file_sz = {}, mem_sz = {}",
            dest as u64,
            src as u64,
            file_sz,
            mem_sz
        );

        unsafe {
            info!("PT_LOAD copy: about to copy_nonoverlapping");
            ptr::copy_nonoverlapping(src, dest, file_sz);
            info!("PT_LOAD copy: copy_nonoverlapping done");

            if mem_sz > file_sz {
                info!(
                    "PT_LOAD zero: dest = {:#x}, count = {}",
                    dest.add(file_sz) as u64,
                    mem_sz - file_sz
                );
                ptr::write_bytes(dest.add(file_sz), 0, mem_sz - file_sz);
                info!("PT_LOAD zero: write_bytes done");
            }
        }
    }

    info!("Jumping to kernel entry = {:#x}", entry_addr);

    unsafe {
    let mut port = x86_64::instructions::port::Port::new(0x3F8);
    port.write(b'J');   // already there
    port.write(b'\n');

    // NEW: dump entry_addr low bytes so we know what we're calling
    let val = entry_addr as u64;
    for shift in (0..16).step_by(4) {
        let nibble = ((val >> shift) & 0xF) as u8;
        let ch = match nibble {
            0..=9 => b'0' + nibble,
            10..=15 => b'a' + (nibble - 10),
            _ => b'?'
        };
        port.write(ch);
    }
    port.write(b'\n');
}

    type KernelEntry = extern "sysv64" fn(&'static mut BulldogBootInfo) -> !;

    let entry: KernelEntry = unsafe { mem::transmute(entry_addr) };

    let boot_info_static: &'static mut BulldogBootInfo =
        unsafe { &mut *(boot_info as *mut BulldogBootInfo) };

    unsafe {
        let stack_base = (&raw mut KERNEL_STACK.0 as *mut u8) as u64;
        let stack_top  = (&raw mut KERNEL_STACK.0 as *mut u8).add(100 * 1024) as u64;

        info!("jump_to_kernel: KERNEL_STACK base = {:#x}", stack_base);
        info!("jump_to_kernel: KERNEL_STACK end  = {:#x}", stack_top);

unsafe {
    let mut port = x86_64::instructions::port::Port::new(0x3F8);
    port.write(b'J');
    port.write(b'\n');
}



type KernelEntry = extern "sysv64" fn(&'static mut BulldogBootInfo) -> !;

let entry: KernelEntry = unsafe { core::mem::transmute(entry_addr) };

let boot_info_static: &'static mut BulldogBootInfo =
    unsafe { &mut *(boot_info as *mut BulldogBootInfo) };

unsafe {
    let stack_base = (&raw mut KERNEL_STACK.0 as *mut u8) as u64;
    let stack_top  = (&raw mut KERNEL_STACK.0 as *mut u8).add(100 * 1024) as u64;

    info!("jump_to_kernel: KERNEL_STACK base = {:#x}", stack_base);
    info!("jump_to_kernel: KERNEL_STACK end  = {:#x}", stack_top);

    // Set RSP and then just call via Rust
    core::arch::asm!(
        "mov rsp, {stack}",
        "and rsp, -16",
        "sub rsp, 8",
        stack = in(reg) stack_top,
        options(nostack)
    );

    // Now do a normal Rust call
    entry(boot_info_static);
}

    }
}

// ============================================================
//  ACPI helpers
// ============================================================

pub fn find_rsdp() -> Option<usize> {
    let mut rsdp = None;

    system::with_config_table(|entries| {
        for entry in entries {
            if entry.guid == ConfigTableEntry::ACPI2_GUID {
                rsdp = Some(entry.address as usize);
            }
        }
    });

    rsdp
}

#[repr(C, packed)]
struct RsdpV1 {
    signature: [u8; 8],
    checksum: u8,
    oem_id: [u8; 6],
    revision: u8,
    rsdt_address: u32,
}

pub fn acpi_revision(rsdp_addr: usize) -> Option<u8> {
    if rsdp_addr == 0 {
        return None;
    }

    let rsdp = unsafe { &*(rsdp_addr as *const RsdpV1) };
    Some(rsdp.revision)
}

// ============================================================
//  Minimal paging bootstrap
// ============================================================



pub struct BootFrameAllocator {
    frames: Vec<PhysFrame>,
    next: usize,
}

impl BootFrameAllocator {
    pub fn new(memory_map: &MemoryMapOwned) -> Self {
        let mut frames = Vec::new();

        for desc in memory_map.entries() {
            if desc.ty != MemoryType::CONVENTIONAL {
                continue;
            }

            let start = desc.phys_start;
            let end = start + desc.page_count * 4096;

            for addr in (start..end).step_by(4096) {
                if addr < 0x10000 {
                    continue;
                }
                frames.push(PhysFrame::containing_address(PhysAddr::new(addr)));
            }
        }

        BootFrameAllocator { frames, next: 0 }
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        if self.next >= self.frames.len() {
            None
        } else {
            let frame = self.frames[self.next];
            self.next += 1;
            Some(frame)
        }
    }
}

unsafe fn create_empty_pml4(alloc: &mut BootFrameAllocator) -> PhysFrame {
    let frame = alloc.allocate_frame().expect("No frame for PML4");
    let ptr = frame.start_address().as_u64() as *mut PageTable;
    ptr.write(PageTable::new());
    frame
}

unsafe fn identity_map_boot_region(
    mapper: &mut OffsetPageTable<'static>,
    alloc: &mut BootFrameAllocator,
    start: u64,
    end: u64,
) {
    use x86_64::structures::paging::{Page, Size4KiB};

    for addr in (start..end).step_by(4096) {
        let phys = PhysAddr::new(addr);
        let virt = VirtAddr::new(addr);

        let page: Page<Size4KiB> = Page::containing_address(virt);
        let frame = PhysFrame::containing_address(phys);

        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;

        mapper
            .map_to(page, frame, flags, alloc)
            .expect("identity map failed")
            .flush();
    }
}

unsafe fn map_hhdm_region(
    mapper: &mut OffsetPageTable<'static>,
    alloc: &mut BootFrameAllocator,
    phys_offset: VirtAddr,
    start: u64,
    end: u64,
) {
    use x86_64::structures::paging::{Page, Size4KiB};

    for addr in (start..end).step_by(4096) {
        let phys = PhysAddr::new(addr);
        let virt = phys_offset + addr;

        let page: Page<Size4KiB> = Page::containing_address(virt);
        let frame = PhysFrame::containing_address(phys);

        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;

        mapper
            .map_to(page, frame, flags, alloc)
            .expect("hhdm map failed")
            .flush();
    }
}

use x86_64::instructions::interrupts;

pub unsafe fn init_paging_and_switch_cr3(
    memory_map: &MemoryMapOwned,
    phys_offset: VirtAddr,
    _kernel_ptr: *mut u8,
    _kernel_len: usize,
) -> PhysAddr {
    use x86_64::registers::control::Cr3;

    info!("init_paging_and_switch_cr3: entered (HHDM-only)");

    interrupts::disable();
    info!("init_paging_and_switch_cr3: interrupts disabled");

    let mut alloc = BootFrameAllocator::new(memory_map);
    info!("init_paging_and_switch_cr3: BootFrameAllocator created");

    // Use the CURRENT active PML4 instead of creating a new one
    let (current_pml4_frame, _) = Cr3::read();
    info!(
        "init_paging_and_switch_cr3: current PML4 frame at {:#x}",
        current_pml4_frame.start_address().as_u64()
    );

    // Assume identity mapping for low memory in current tables
    let pml4_ptr = current_pml4_frame.start_address().as_u64() as *mut PageTable;
    let mut mapper = OffsetPageTable::new(&mut *pml4_ptr, VirtAddr::new(0));
    info!("init_paging_and_switch_cr3: OffsetPageTable over existing PML4");

    let start: u64 = 0;
    let end: u64 = 0x1_0000_0000; // 4 GiB

    info!(
        "init_paging_and_switch_cr3: mapping HHDM region {:#x}..{:#x} at offset {:#x}",
        start,
        end,
        phys_offset.as_u64()
    );

    map_hhdm_region(&mut mapper, &mut alloc, phys_offset, start, end);
    info!("init_paging_and_switch_cr3: map_hhdm_region done");

    // Do NOT change CR3 yet; just return the current PML4
    current_pml4_frame.start_address()
}



/// Create a new PML4 by copying the identity-mapped PML4 entries
/// from the currently active UEFI page table.
/// No HHDM yet. No kernel mappings. Just identity.
pub unsafe fn create_identity_pml4_copy(
    frame_alloc: &mut impl FrameAllocator<Size4KiB>,
    max_phys_addr: u64,
    framebuffer_addr: u64,
    framebuffer_len: u64,
) -> PhysFrame {
    // 1. Read current CR3
    let (current_pml4_frame, _) = Cr3::read();
    let current_pml4_ptr =
        current_pml4_frame.start_address().as_u64() as *const PageTable;
    let current_pml4 = &*current_pml4_ptr;

    // 2. Allocate a new PML4 frame
    let new_pml4_frame = frame_alloc.allocate_frame().expect("no frame for new PML4");
    let new_pml4_ptr =
        new_pml4_frame.start_address().as_u64() as *mut PageTable;

    // 3. Zero it
    new_pml4_ptr.write(PageTable::new());
    let new_pml4 = &mut *new_pml4_ptr;

    // 4. Copy PML4 entries that cover identity-mapped RAM
    let end = VirtAddr::new(max_phys_addr - 1);
    for p4 in 0..=usize::from(end.p4_index()) {
        new_pml4[p4] = current_pml4[p4].clone();
    }

    // 5. Copy PML4 entries that cover the framebuffer
    let fb_start = VirtAddr::new(framebuffer_addr);
    let fb_end = fb_start + framebuffer_len;
    for p4 in usize::from(fb_start.p4_index())..=usize::from(fb_end.p4_index()) {
        new_pml4[p4] = current_pml4[p4].clone();
    }

    new_pml4_frame
}


/// Build Bulldog's first paging setup:
/// - Identity map 0..4 GiB using 2 MiB pages
/// - No HHDM, no higher-half, no UEFI PML4
/// Returns the PML4 frame to load into CR3.
fn alloc_low_frame(alloc: &mut BootFrameAllocator) -> PhysFrame {
    loop {
        let frame = alloc.allocate_frame().expect("out of frames");
        let addr = frame.start_address().as_u64();
        // keep only frames below 1 GiB
        if addr < 0x4000_0000 {
            return frame;
        }
        // otherwise skip and try next
    }
}

pub unsafe fn create_bulldog_identity_paging(
    frame_alloc: &mut BootFrameAllocator,
) -> PhysFrame {
    use x86_64::structures::paging::PageTableFlags;

    // 1) Allocate PML4, PDPT, and 4 PDs (for 4 GiB), all from low memory
    let pml4_frame = alloc_low_frame(frame_alloc);
    let pdpt_frame = alloc_low_frame(frame_alloc);

    let pd0_frame = alloc_low_frame(frame_alloc);
    let pd1_frame = alloc_low_frame(frame_alloc);
    let pd2_frame = alloc_low_frame(frame_alloc);
    let pd3_frame = alloc_low_frame(frame_alloc);

    // 2) Zero them via (assumed) identity mapping
    let pml4 = pml4_frame.start_address().as_u64() as *mut PageTable;
    let pdpt = pdpt_frame.start_address().as_u64() as *mut PageTable;
    let pd0  = pd0_frame.start_address().as_u64() as *mut PageTable;
    let pd1  = pd1_frame.start_address().as_u64() as *mut PageTable;
    let pd2  = pd2_frame.start_address().as_u64() as *mut PageTable;
    let pd3  = pd3_frame.start_address().as_u64() as *mut PageTable;

    pml4.write(PageTable::new());
    pdpt.write(PageTable::new());
    pd0.write(PageTable::new());
    pd1.write(PageTable::new());
    pd2.write(PageTable::new());
    pd3.write(PageTable::new());

    let pml4 = &mut *pml4;
    let pdpt = &mut *pdpt;
    let pd0  = &mut *pd0;
    let pd1  = &mut *pd1;
    let pd2  = &mut *pd2;
    let pd3  = &mut *pd3;

    // 3) Wire PML4[0] -> PDPT
    let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
    pml4[0].set_frame(pdpt_frame, flags);

    // 4) Wire PDPT[0..3] -> PDs
    pdpt[0].set_frame(pd0_frame, flags);
    pdpt[1].set_frame(pd1_frame, flags);
    pdpt[2].set_frame(pd2_frame, flags);
    pdpt[3].set_frame(pd3_frame, flags);

    // 5) Fill PDs with 2 MiB identity entries
    let huge_flags = flags | PageTableFlags::HUGE_PAGE;

    // PD0: 0..1 GiB
    for (i, entry) in pd0.iter_mut().enumerate() {
        let phys = (i as u64) * 0x20_0000; // 2 MiB
        entry.set_addr(PhysAddr::new(phys), huge_flags);
    }

    // PD1: 1..2 GiB
    for (i, entry) in pd1.iter_mut().enumerate() {
        let phys = 0x4000_0000 + (i as u64) * 0x20_0000;
        entry.set_addr(PhysAddr::new(phys), huge_flags);
    }

    // PD2: 2..3 GiB
    for (i, entry) in pd2.iter_mut().enumerate() {
        let phys = 0x8000_0000 + (i as u64) * 0x20_0000;
        entry.set_addr(PhysAddr::new(phys), huge_flags);
    }

    // PD3: 3..4 GiB
    for (i, entry) in pd3.iter_mut().enumerate() {
        let phys = 0xC000_0000 + (i as u64) * 0x20_0000;
        entry.set_addr(PhysAddr::new(phys), huge_flags);
    }

    pml4_frame
}

pub unsafe fn create_bootstrap_identity_pml4(
    frame_alloc: &mut BootFrameAllocator,
) -> PhysFrame {
    use x86_64::structures::paging::PageTableFlags;

    // Allocate just a PML4 and a single PDPT + PD for now (0..1 GiB)
    let pml4_frame = alloc_low_frame(frame_alloc);
    let pdpt_frame = alloc_low_frame(frame_alloc);
    let pd0_frame  = alloc_low_frame(frame_alloc);

    let pml4_ptr = pml4_frame.start_address().as_u64() as *mut PageTable;
    let pdpt_ptr = pdpt_frame.start_address().as_u64() as *mut PageTable;
    let pd0_ptr  = pd0_frame.start_address().as_u64() as *mut PageTable;

    // Zero them (we still rely on low‑mem identity here, but only for a tiny set)
    pml4_ptr.write(PageTable::new());
    pdpt_ptr.write(PageTable::new());
    pd0_ptr.write(PageTable::new());

    let pml4 = &mut *pml4_ptr;
    let pdpt = &mut *pdpt_ptr;
    let pd0  = &mut *pd0_ptr;

    let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;

    // PML4[0] → PDPT
    pml4[0].set_frame(pdpt_frame, flags);

    // PDPT[0] → PD0
    pdpt[0].set_frame(pd0_frame, flags);

    // PD0: 0..1 GiB, 2 MiB pages
    let huge_flags = flags | PageTableFlags::HUGE_PAGE;
    for (i, entry) in pd0.iter_mut().enumerate() {
        let phys = (i as u64) * 0x20_0000; // 2 MiB
        entry.set_addr(PhysAddr::new(phys), huge_flags);
    }

    pml4_frame
}




   





