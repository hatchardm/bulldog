// boot.rs
use crate::console::Console;
use uefi::system;
use uefi::table::cfg::ConfigTableEntry;
use uefi::boot::{self, MemoryType};
use uefi::mem::memory_map::MemoryMap;
use log::info;
use core::slice;

use boot_proto::{
    BootInfo as BulldogBootInfo,
    MemoryRegion as BulldogMemoryRegion,
    MemoryRegionKind as BulldogMemoryRegionKind,
};





// Enough for your current 107 entries with headroom
static mut MEMORY_REGIONS: [core::mem::MaybeUninit<BulldogMemoryRegion>; 256] =
    [const { core::mem::MaybeUninit::uninit() }; 256];

pub fn load_kernel(console: &mut Console) -> Result<(*mut u8, usize), ()> {
    use core::slice;
    use uefi::boot;
    use uefi::proto::media::file::{
        File, FileMode, FileAttribute, FileType, FileInfo,
    };
    use uefi::CString16;

    console.write_str("Opening kernel file...\n");

    // 1) Get the image handle for this bootloader
    let image = boot::image_handle();

    // 2) Get the filesystem for this image (must be mutable)
    let mut fs = boot::get_image_file_system(image)
    .expect("get_image_file_system failed");

    // 3) Open the root directory
    let mut root = fs.open_volume()
    .expect("open_volume failed");


    // 4) Build UEFI path as CStr16
    let kernel_path = CString16::try_from("\\EFI\\BOOT\\kernel.elf")
        .expect("invalid kernel path");

    // 5) Open \EFI\BOOT\kernel
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

    // 6) Get file size via get_info(buffer)
    let mut info_buf = [0u8; 512];
    let info = kernel_file
        .get_info::<FileInfo>(&mut info_buf)
        .expect("get_info failed");

    let file_size = info.file_size() as usize;

    // 7) Allocate pool for the kernel file
    let buf_nn = boot::allocate_pool(MemoryType::LOADER_DATA, file_size)
        .expect("allocate_pool failed");
    let buf_ptr = buf_nn.as_ptr();

    let buf_slice = unsafe { slice::from_raw_parts_mut(buf_ptr, file_size) };

    // 8) Read kernel into memory
    kernel_file
        .read(buf_slice)
        .expect("kernel read failed");

    console.write_str("Kernel loaded into memory.\n");

    Ok((buf_ptr, file_size))
}




// jump_to_kernel now takes the REAL BootInfo

pub fn jump_to_kernel(
    kernel_ptr: *mut u8,
    kernel_len: usize,
    boot_info: &mut BulldogBootInfo,
) -> ! {
    use core::{mem, ptr, slice};

    log::info!(
        "jump_to_kernel entered: ptr = {:#x}, len = {}",
        kernel_ptr as u64,
        kernel_len
    );

    let elf = unsafe { slice::from_raw_parts(kernel_ptr as *const u8, kernel_len) };

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

    let ehdr = unsafe { &*(elf.as_ptr() as *const Elf64Ehdr) };
    let ph_base = unsafe { elf.as_ptr().add(ehdr.e_phoff as usize) as *const Elf64Phdr };

    let entry_addr = ehdr.e_entry;
    log::info!(
        "ELF header: entry = {:#x}, phoff = {:#x}, phnum = {}",
        entry_addr,
        ehdr.e_phoff,
        ehdr.e_phnum
    );

    // Load PT_LOAD segments
    for i in 0..ehdr.e_phnum {
        let ph = unsafe { &*ph_base.add(i as usize) };
        if ph.p_type != PT_LOAD {
            continue;
        }

        let dest = ph.p_vaddr as *mut u8;
        let src = unsafe { elf.as_ptr().add(ph.p_offset as usize) };
        let file_sz = ph.p_filesz as usize;
        let mem_sz = ph.p_memsz as usize;

        unsafe {
            ptr::copy_nonoverlapping(src, dest, file_sz);
            if mem_sz > file_sz {
                ptr::write_bytes(dest.add(file_sz), 0, mem_sz - file_sz);
            }
        }
    }

    log::info!("about to jump to kernel entry = {:#x}", entry_addr as u64);

    let entry: extern "C" fn(&'static mut BulldogBootInfo) -> ! =
        unsafe { mem::transmute(entry_addr) };

    let boot_info_static: &'static mut BulldogBootInfo =
        unsafe { &mut *(boot_info as *mut BulldogBootInfo) };

    entry(boot_info_static)
}






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

pub fn log_memory_map() {
    // Ask UEFI for the current memory map; it returns a MemoryMapOwned
    let memory_map = boot::memory_map(MemoryType::LOADER_DATA)
        .expect("Failed to retrieve UEFI memory map");

    let mut count = 0usize;

    // Iterate over descriptors via .entries()
    for _desc in memory_map.entries() {
        count += 1;
    }

    info!("UEFI memory map entries: {}", count);
}

pub fn fill_memory_regions(boot_info: &mut BulldogBootInfo) {
    let memory_map = boot::memory_map(MemoryType::LOADER_DATA)
        .expect("Failed to retrieve UEFI memory map");

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
        slice::from_raw_parts(ptr, idx)
    };

    boot_info.memory_regions = slice;
}



   





