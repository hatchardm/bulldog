use crate::console::Console;
use uefi::system;
use uefi::table::cfg::ConfigTableEntry;
use uefi::boot::{self, MemoryType};
use uefi::mem::memory_map::MemoryMapOwned;
use uefi::mem::memory_map::MemoryMap;
use log::info;
use core::slice;
use boot_proto::{
    BootInfo,
    MemoryRegion,
    MemoryRegionKind,
};

extern crate alloc;

#[repr(align(16))]
struct AlignedKernelStack([u8; 100 * 1024]);

#[unsafe(no_mangle)]
static mut KERNEL_STACK: AlignedKernelStack = AlignedKernelStack([0; 100 * 1024]);

// ============================================================
//  ELF definitions
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

static mut MEMORY_REGIONS: [core::mem::MaybeUninit<MemoryRegion>; 256] =
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

pub fn fill_memory_regions_from_map(
    boot_info: &mut BootInfo,
    memory_map: &MemoryMapOwned,
) {
    let mut idx = 0usize;

    for desc in memory_map.entries() {
        if idx >= boot_info.memory_regions_buffer.len() {
            break;
        }

        let start = desc.phys_start;
        let len_bytes = desc.page_count * 4096;
        let end = start + len_bytes;

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

            // 🔹 FIX: apply relocation inside the ELF buffer
            let loc = unsafe {
                elf.as_ptr().add(rela.r_offset as usize) as *mut u64
            };

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

    // Track kernel range across all loadable segments (using PHYSICAL addresses)
let mut kernel_phys_start = u64::MAX;
let mut kernel_phys_end   = 0u64;

for i in 0..ehdr.e_phnum {
    let ph_ptr = unsafe { ph_base.add(i as usize) };
    let ph = unsafe { &*ph_ptr };

    // Only load PT_LOAD (and your custom 0x1000 if needed)
    if ph.p_type != PT_LOAD && ph.p_type != 0x1000 {
        continue;
    }

    // COPY SEGMENT TO PHYSICAL ADDRESS, NOT p_vaddr
    let dest = ph.p_paddr as *mut u8;
    let src  = unsafe { elf.as_ptr().add(ph.p_offset as usize) };

    let file_sz = ph.p_filesz as usize;
    let mem_sz  = ph.p_memsz  as usize;

    // Compute physical segment range
    let seg_start = ph.p_paddr;
    let seg_end   = ph.p_paddr + if mem_sz != 0 {
        mem_sz as u64
    } else {
        file_sz as u64
    };





    // Track kernel physical range
    if seg_start < kernel_phys_start {
        kernel_phys_start = seg_start;
    }
    if seg_end > kernel_phys_end {
        kernel_phys_end = seg_end;
    }

    // Copy file portion
    unsafe {
        core::ptr::copy_nonoverlapping(src, dest, file_sz);

        // Zero BSS portion
        if mem_sz > file_sz {
            core::ptr::write_bytes(dest.add(file_sz), 0, mem_sz - file_sz);
        }
    }
}

       

    // Store kernel range and entry in BootInfo
    boot_info.kernel_phys_start = kernel_phys_start;
    boot_info.kernel_phys_end   = kernel_phys_end;
    boot_info.kernel_entry_phys = entry_addr;


    info!("Jumping to kernel entry = {:#x}", entry_addr);




    type KernelEntry = extern "sysv64" fn(&mut BootInfo) -> !;

    let entry: KernelEntry = unsafe { mem::transmute(entry_addr) };

    



    let boot_info_static: &'static mut BootInfo =
        unsafe { &mut *(boot_info as *mut BootInfo) };

    unsafe {
        let stack_base = (&raw mut KERNEL_STACK.0 as *mut u8) as u64;
        let stack_top  = (&raw mut KERNEL_STACK.0 as *mut u8).add(100 * 1024) as u64;

        info!("jump_to_kernel: KERNEL_STACK base = {:#x}", stack_base);
        info!("jump_to_kernel: KERNEL_STACK end  = {:#x}", stack_top);

        
    



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




