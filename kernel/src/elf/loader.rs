use crate::elf::types::*;
use core::mem::{size_of, align_of};
use core::slice;



#[derive(Debug)]
pub enum ElfError {
    BadMagic,
    BadClass,
    BadEndian,
    BadMachine,
    BadType,
    BadHeaderSize,
    BadPhSize,
    PhOutOfBounds,
}

pub fn validate_elf_header(hdr: &Elf64_Ehdr) -> Result<(), ElfError> {
    // Magic bytes: 0x7F 'E' 'L' 'F'
    if hdr.e_ident[0] != ELFMAG0 ||
       hdr.e_ident[1] != ELFMAG1 ||
       hdr.e_ident[2] != ELFMAG2 ||
       hdr.e_ident[3] != ELFMAG3 {
        return Err(ElfError::BadMagic);
    }

    // Must be 64-bit ELF
    if hdr.e_ident[4] != ELFCLASS64 {
        return Err(ElfError::BadClass);
    }

    // Must be little-endian
    if hdr.e_ident[5] != ELFDATA2LSB {
        return Err(ElfError::BadEndian);
    }

    // Must be x86_64
    if hdr.e_machine != EM_X86_64 {
        return Err(ElfError::BadMachine);
    }

    // Must be an executable file
    if hdr.e_type != ET_EXEC {
        return Err(ElfError::BadType);
    }

    // Header size must match our struct
    if hdr.e_ehsize as usize != size_of::<Elf64_Ehdr>() {
        return Err(ElfError::BadHeaderSize);
    }

    // Program header entry size must match our struct
    if hdr.e_phentsize as usize != size_of::<Elf64_Phdr>() {
        return Err(ElfError::BadPhSize);
    }

    Ok(())
}

pub fn get_program_headers<'a>(
    elf_data: &'a [u8],
    hdr: &Elf64_Ehdr,
) -> Result<&'a [Elf64_Phdr], ElfError> {
    let phoff = hdr.e_phoff as usize;
    let phentsize = hdr.e_phentsize as usize;
    let phnum = hdr.e_phnum as usize;

    let total_size = phentsize * phnum;

    // Bounds check: ensure the program header table fits in the ELF file
    if phoff + total_size > elf_data.len() {
        return Err(ElfError::PhOutOfBounds);
    }

    // Safety: we know the bytes are in-bounds and properly sized
    let ptr = unsafe {
        elf_data.as_ptr().add(phoff) as *const Elf64_Phdr
    };

    let slice = unsafe {
        slice::from_raw_parts(ptr, phnum)
    };

    Ok(slice)
}

pub struct SegmentFlags {
    pub executable: bool,
    pub writable:   bool,
    pub readable:   bool,
}

pub fn load_segments<F>(
    elf_data: &[u8],
    hdr: &Elf64_Ehdr,
    phdrs: &[Elf64_Phdr],
    mut map_segment: F,
) -> Result<(), ElfError>
where
    // vaddr, mem_size, file_bytes, flags
    F: FnMut(u64, usize, &[u8], &SegmentFlags) -> Result<(), ()>,
{
    for ph in phdrs {
        if ph.p_type != PT_LOAD {
            continue;
        }

        let file_offset = ph.p_offset as usize;
        let file_size   = ph.p_filesz as usize;
        let vaddr       = ph.p_vaddr;
        let mem_size    = ph.p_memsz as usize;

        // Basic bounds check for file-backed part
        if file_offset + file_size > elf_data.len() {
            return Err(ElfError::PhOutOfBounds);
        }

        let file_bytes = &elf_data[file_offset .. file_offset + file_size];

        let flags = SegmentFlags {
            executable: (ph.p_flags & 0x1) != 0, // PF_X
            writable:   (ph.p_flags & 0x2) != 0, // PF_W
            readable:   (ph.p_flags & 0x4) != 0, // PF_R
        };

        // Mapper now knows:
        // - vaddr: where to map
        // - mem_size: total in-memory size (file + zero-fill)
        // - file_bytes: file-backed portion
        // - flags: R/W/X
        map_segment(vaddr, mem_size, file_bytes, &flags)
            .map_err(|_| ElfError::BadPhSize)?;
    }

    Ok(())
}