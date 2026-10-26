//! Bootloader core – UEFI 0.41
//!
//! Responsibilities:
//! - Load kernel.elf and fat.img from the ESP
//! - Fill BootInfo (framebuffer, memory map, FAT location)
//! - Apply ELF relocations
//! - Jump to the kernel with a clean stack

#![allow(dead_code)]

use core::slice;
use log::info;
use uefi::prelude::*;
use uefi::boot::{self, AllocateType};
use uefi::fs::FileSystem;
use uefi::mem::memory_map::{MemoryMap, MemoryType};
use uefi::CString16;
use uefi::proto::media::file::File;
use boot_proto::{BootInfo, MemoryRegion, MemoryRegionKind, PixelFormat};
use crate::console::Console;


// ============================================================
//  Kernel stack (16-byte aligned)
// ============================================================

#[repr(align(16))]
struct AlignedKernelStack([u8; 100 * 1024]);

#[unsafe(no_mangle)]
static mut KERNEL_STACK: AlignedKernelStack = AlignedKernelStack([0; 100 * 1024]);

// ============================================================
//  ELF structures
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
//  Load kernel.elf + fat.img
// ============================================================

/// Returns (kernel_ptr, kernel_size, fat_phys_addr, fat_size)
pub fn load_kernel(console: &mut Console) -> Result<(*mut u8, usize, u64, usize), Status> {
    console.write_str("Opening filesystem...\n");

    let sfs = boot::get_image_file_system(boot::image_handle())
        .map_err(|e| e.status())?;
    let mut fs = FileSystem::new(sfs);

    // ---------- kernel.elf (small – high-level API is fine) ----------
    console.write_str("Loading kernel.elf...\n");

    let kernel_path = CString16::try_from("\\EFI\\BOOT\\kernel.elf")
        .map_err(|_| Status::INVALID_PARAMETER)?;

    let kernel_data = fs
        .read(kernel_path.as_ref())
        .map_err(|_| {
            console.write_str("Failed to read kernel.elf\n");
            Status::LOAD_ERROR
        })?;

    let kernel_size = kernel_data.len();
    let kernel_ptr = boot::allocate_pool(MemoryType::LOADER_DATA, kernel_size)
        .map_err(|e| e.status())?
        .as_ptr();

    unsafe {
        core::ptr::copy_nonoverlapping(kernel_data.as_ptr(), kernel_ptr, kernel_size);
    }

    console.write_str("Kernel loaded.\n");

    // ---------- fat.img (large – read directly into pages) ----------
    console.write_str("Loading fat.img...\n");

    // Drop the high-level FileSystem so we can re-open the protocol
    drop(fs);

    let mut sfs = boot::get_image_file_system(boot::image_handle())
        .map_err(|e| e.status())?;

    let mut root = sfs.open_volume().map_err(|e| e.status())?;

    let fat_path = CString16::try_from("\\EFI\\BOOT\\fat.img")
        .map_err(|_| Status::INVALID_PARAMETER)?;

    let mut fat_file = root
        .open(
            &fat_path,
            uefi::proto::media::file::FileMode::Read,
            uefi::proto::media::file::FileAttribute::empty(),
        )
        .map_err(|e| e.status())?
        .into_regular_file()
        .ok_or(Status::LOAD_ERROR)?;

    // Get file size
    let mut info_buf = [0u8; 512];
    let info = fat_file
        .get_info::<uefi::proto::media::file::FileInfo>(&mut info_buf)
        .map_err(|e| e.status())?;
    let fat_size = info.file_size() as usize;

    let pages = (fat_size + 0xFFF) / 0x1000;
    info!("fat_size = {}, pages = {}", fat_size, pages);

    let fat_phys = boot::allocate_pages(
        AllocateType::AnyPages,
        MemoryType::LOADER_DATA,
        pages,
    )
    .map_err(|e| e.status())?
    .as_ptr() as u64;

    info!("FAT image at {:#x}", fat_phys);

    // Read directly into the allocated buffer (no large Vec)
    let fat_slice = unsafe {
        core::slice::from_raw_parts_mut(fat_phys as *mut u8, fat_size)
    };

    fat_file.read(fat_slice).map_err(|e| e.status())?;

    console.write_str("FAT image loaded.\n");

    Ok((kernel_ptr, kernel_size, fat_phys, fat_size))
}

// ============================================================
//  ELF relocation pass
// ============================================================

fn apply_relocations(elf: &[u8]) {
    let ehdr = unsafe { &*(elf.as_ptr() as *const Elf64Ehdr) };
    let sh_base = unsafe { elf.as_ptr().add(ehdr.e_shoff as usize) as *const Elf64Shdr };

    for i in 0..ehdr.e_shnum {
        let sh = unsafe { &*sh_base.add(i as usize) };
        if sh.sh_type != SHT_RELA {
            continue;
        }

        let count = (sh.sh_size / sh.sh_entsize) as usize;
        let rela_slice = unsafe {
            slice::from_raw_parts(
                elf.as_ptr().add(sh.sh_offset as usize) as *const Elf64Rela,
                count,
            )
        };

        for rela in rela_slice {
            let r_type = (rela.r_info & 0xff) as u32;
            if r_type != R_X86_64_RELATIVE {
                continue;
            }

            let loc = unsafe { elf.as_ptr().add(rela.r_offset as usize) as *mut u64 };
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
    boot_info: &mut BootInfo,
) -> ! {
    info!(
        "jump_to_kernel: ptr = {:#x}, len = {}",
        kernel_ptr as u64, kernel_len
    );

    let elf = unsafe { slice::from_raw_parts(kernel_ptr as *const u8, kernel_len) };
    let ehdr = unsafe { &*(elf.as_ptr() as *const Elf64Ehdr) };
    let ph_base = unsafe { elf.as_ptr().add(ehdr.e_phoff as usize) as *const Elf64Phdr };

    let entry_addr = ehdr.e_entry;

    info!(
        "ELF: entry = {:#x}, phoff = {:#x}, phnum = {}",
        entry_addr, ehdr.e_phoff, ehdr.e_phnum
    );

    apply_relocations(elf);

    let mut kernel_phys_start = u64::MAX;
    let mut kernel_phys_end = 0u64;

    for i in 0..ehdr.e_phnum {
        let ph = unsafe { &*ph_base.add(i as usize) };

        if ph.p_type != PT_LOAD && ph.p_type != 0x1000 {
            continue;
        }

        let dest = ph.p_paddr as *mut u8;
        let src = unsafe { elf.as_ptr().add(ph.p_offset as usize) };
        let file_sz = ph.p_filesz as usize;
        let mem_sz = ph.p_memsz as usize;

        let seg_start = ph.p_paddr;
        let seg_end = ph.p_paddr + if mem_sz != 0 { mem_sz as u64 } else { file_sz as u64 };

        if seg_start < kernel_phys_start {
            kernel_phys_start = seg_start;
        }
        if seg_end > kernel_phys_end {
            kernel_phys_end = seg_end;
        }

        unsafe {
            core::ptr::copy_nonoverlapping(src, dest, file_sz);
            if mem_sz > file_sz {
                core::ptr::write_bytes(dest.add(file_sz), 0, mem_sz - file_sz);
            }
        }
    }

    boot_info.kernel_phys_start = kernel_phys_start;
    boot_info.kernel_phys_end = kernel_phys_end;
    boot_info.kernel_entry_phys = entry_addr;

    info!("Jumping to kernel entry = {:#x}", entry_addr);

    type KernelEntry = extern "sysv64" fn(&mut BootInfo) -> !;
    let entry: KernelEntry = unsafe { core::mem::transmute(entry_addr) };

    let boot_info_static: &'static mut BootInfo =
        unsafe { &mut *(boot_info as *mut BootInfo) };

    unsafe {
        let stack_top = (&raw mut KERNEL_STACK.0 as *mut u8).add(100 * 1024) as u64;

        core::arch::asm!(
            "mov rsp, {stack}",
            "and rsp, -16",
            "sub rsp, 8",
            stack = in(reg) stack_top,
            options(nostack)
        );

        entry(boot_info_static);
    }
}

// ============================================================
//  fill_boot_info
// ============================================================

pub fn fill_boot_info(
    boot_info: &mut BootInfo,
    fb: &crate::framebuffer::Framebuffer,
    pixel_format: uefi::proto::console::gop::PixelFormat,
    fat_phys: u64,
    fat_len: usize,
) {
    // Framebuffer (physical)
    boot_info.framebuffer.addr = fb.ptr as u64;
    boot_info.framebuffer.width = fb.width as u32;
    boot_info.framebuffer.height = fb.height as u32;
    boot_info.framebuffer.stride = fb.stride as u32;
    boot_info.framebuffer.pixel_format = match pixel_format {
        uefi::proto::console::gop::PixelFormat::Rgb => PixelFormat::Rgb,
        uefi::proto::console::gop::PixelFormat::Bgr => PixelFormat::Bgr,
        uefi::proto::console::gop::PixelFormat::Bitmask => PixelFormat::Bitmask,
        uefi::proto::console::gop::PixelFormat::BltOnly => PixelFormat::BltOnly,
    };
    boot_info.framebuffer_present = 1;

    // Higher-half mapping info
    boot_info.physical_memory_offset = crate::paging::HHDM_BASE;
    boot_info.framebuffer_virt = crate::paging::HHDM_BASE + fb.ptr as u64;

    // FAT image
    boot_info.fat_phys_addr = fat_phys;
    boot_info.fat_size = fat_len as u64;

    // Memory map
    let mmap = boot::memory_map(MemoryType::LOADER_DATA)
        .expect("Failed to retrieve memory map");

    let mut idx = 0usize;
    for desc in mmap.entries() {
        if idx >= boot_info.memory_regions_buffer.len() {
            break;
        }

        let start = desc.phys_start;
        let end = start + desc.page_count * 4096;

        let kind = match desc.ty {
            MemoryType::CONVENTIONAL => MemoryRegionKind::Usable,
            MemoryType::ACPI_RECLAIM => MemoryRegionKind::Acpi,
            MemoryType::MMIO | MemoryType::MMIO_PORT_SPACE => MemoryRegionKind::Mmio,
            _ => MemoryRegionKind::Reserved,
        };

        boot_info.memory_regions_buffer[idx] = MemoryRegion { start, end, kind };
        idx += 1;
    }

    boot_info.memory_regions = boot_info.memory_regions_buffer.as_ptr();
    boot_info.memory_region_count = idx;
}

// ============================================================
//  Optional ACPI helper (kept for convenience)
// ============================================================

#[repr(C, packed)]
struct RsdpV1 {
    signature: [u8; 8],
    checksum: u8,
    oem_id: [u8; 6],
    revision: u8,
    rsdt_address: u32,
}

pub fn find_rsdp() -> Option<usize> {
    // In 0.41 the config table is accessed via the system table helpers
    // if you need it later. For now this is a stub that can be filled
    // once you decide to pass RSDP to the kernel.
    None
}