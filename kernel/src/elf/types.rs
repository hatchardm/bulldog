#![allow(non_camel_case_types)]
#![allow(dead_code)]

use core::mem::size_of;

pub const EI_NIDENT: usize = 16;

// --- ELF Header (64-bit) ---
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Elf64_Ehdr {
    pub e_ident: [u8; EI_NIDENT],
    pub e_type:   u16,
    pub e_machine: u16,
    pub e_version: u32,
    pub e_entry:   u64,
    pub e_phoff:   u64,
    pub e_shoff:   u64,
    pub e_flags:   u32,
    pub e_ehsize:  u16,
    pub e_phentsize: u16,
    pub e_phnum:     u16,
    pub e_shentsize: u16,
    pub e_shnum:     u16,
    pub e_shstrndx:  u16,
}

impl Elf64_Ehdr {
    pub const SIZE: usize = size_of::<Elf64_Ehdr>();
}

// --- Program Header (64-bit) ---
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Elf64_Phdr {
    pub p_type:   u32,
    pub p_flags:  u32,
    pub p_offset: u64,
    pub p_vaddr:  u64,
    pub p_paddr:  u64,
    pub p_filesz: u64,
    pub p_memsz:  u64,
    pub p_align:  u64,
}

impl Elf64_Phdr {
    pub const SIZE: usize = size_of::<Elf64_Phdr>();
}

// --- Program header types ---
pub const PT_NULL: u32 = 0;
pub const PT_LOAD: u32 = 1;

// --- ELF magic ---
pub const ELFMAG0: u8 = 0x7F;
pub const ELFMAG1: u8 = b'E';
pub const ELFMAG2: u8 = b'L';
pub const ELFMAG3: u8 = b'F';

// --- ELF class ---
pub const ELFCLASS64: u8 = 2;

// --- ELF data encoding ---
pub const ELFDATA2LSB: u8 = 1;

// --- ELF type ---
pub const ET_EXEC: u16 = 2;

// --- Machine type ---
pub const EM_X86_64: u16 = 62;