## Bulldog Kernel – Bootloader Architecture (UEFI 0.41)

Reviewed: November 2026

### Overview

This document describes the current Bulldog bootloader implementation based on the UEFI 0.41 architecture.

The Bulldog bootloader is responsible for preparing the execution environment required by the kernel before transferring control to `kernel_main()`.

Unlike earlier revisions, the current bootloader performs significantly more than simple ELF loading.

Current responsibilities include:

- GOP framebuffer initialization
- Framebuffer handoff
- FAT image loading
- Memory map collection
- BootInfo construction
- Higher-Half Direct Map (HHDM) setup
- Identity mapping
- Kernel ELF loading
- ELF relocation processing
- Paging activation
- Kernel handoff

This document reflects the implementation in:

    bootloader/src/main.rs
    bootloader/src/boot_mod.rs
    bootloader/src/paging.rs
    boot-proto/src/lib.rs

---

## 1. Boot Sequence

Current boot flow:

    UEFI Firmware
            ↓
    efi_main()
            ↓
    Initialize GOP
            ↓
    Create framebuffer console
            ↓
    Load kernel.elf
            ↓
    Load fat.img
            ↓
    Build BootInfo
            ↓
    ExitBootServices()
            ↓
    Create paging structures
            ↓
    Enable identity map + HHDM
            ↓
    Load CR3
            ↓
    Jump to kernel entry
            ↓
    kernel_main()

All major stages are logged through the bootloader logging system.

---

## 2. UEFI Environment

When Bulldog begins execution:

- CPU is already in 64-bit long mode
- UEFI Boot Services are available
- Graphics Output Protocol (GOP) is available
- Firmware memory map is available
- Paging is active
- Interrupts remain disabled

The bootloader completes all remaining initialization before handing control to the kernel.

---

## 3. Graphics Output Protocol (GOP)

The bootloader initializes the Graphics Output Protocol and retrieves:

    Framebuffer base address
    Framebuffer dimensions
    Framebuffer stride
    Framebuffer pixel format

Supported pixel formats:

    Rgb
    Bgr
    Bitmask
    BltOnly

These values are provided to the kernel through BootInfo.

---

## 4. Bootloader Framebuffer

The bootloader creates a temporary framebuffer abstraction and console.

Primary responsibilities:

    Boot status messages
    Kernel loading diagnostics
    FAT image loading diagnostics
    Boot failure reporting

Framebuffer information tracked:

    Pointer
    Size
    Width
    Height
    Stride

This framebuffer remains active only during bootloader execution.

---

## 5. Loading kernel.elf

The bootloader loads:

    \EFI\BOOT\kernel.elf

from the EFI System Partition.

The loader:

- Reads ELF headers
- Processes program headers
- Copies loadable segments
- Clears BSS regions
- Applies relocations

Kernel metadata recorded in BootInfo:

    kernel_phys_start
    kernel_phys_end
    kernel_entry_phys

---

## 6. Loading fat.img

The bootloader loads:

    \EFI\BOOT\fat.img

from the EFI System Partition.

The image is copied into RAM before kernel startup.

The following values are stored in BootInfo:

    fat_phys_addr
    fat_size

Purpose:

    Initial FAT32 filesystem
    Kernel test files
    User applications
    System data

The kernel later accesses this image through its FAT32 implementation.

---

## 7. BootInfo Handoff

BootInfo is defined in:

    boot-proto/src/lib.rs

The BootInfo structure currently contains:

### Framebuffer Information

    Framebuffer address
    Width
    Height
    Stride
    Pixel format
    Framebuffer virtual address

### Memory Information

    Physical memory offset (HHDM)
    Memory region array
    Memory region count

### Kernel Information

    Kernel physical start
    Kernel physical end
    Kernel entry address

### Filesystem Information

    FAT image physical address
    FAT image size

BootInfo serves as the primary contract between bootloader and kernel.

Any changes must remain synchronized between:

    bootloader
    boot-proto
    kernel

---

## 8. Memory Map Handoff

The bootloader retrieves the UEFI memory map and converts it into Bulldog memory regions.

Supported types:

    Usable
    Reserved
    Acpi
    Mmio

These entries are copied into the BootInfo memory region buffer before kernel entry.

The kernel later uses this information to:

    Construct frame allocators
    Initialize heap allocators
    Map hardware regions
    Validate available memory

---

## 9. ExitBootServices

Before entering the kernel, Bulldog executes:

    ExitBootServices()

This permanently exits the firmware environment.

After this stage:

    UEFI services are unavailable
    Firmware allocations cease
    Memory ownership transfers to Bulldog

The kernel must not invoke UEFI functionality after this point.

---

## 10. Paging Architecture

The bootloader constructs page tables before kernel handoff.

Current implementation:

    4 GiB mapped range
    2 MiB huge pages

Two independent mappings are created.

### 10.1 Identity Mapping

Physical memory is mapped directly:

    virtual = physical

Purpose:

    Early execution
    Boot compatibility
    Physical address accessibility

### 10.2 Higher-Half Direct Map (HHDM)

The bootloader also creates a higher-half direct map.

Base address:

    0xFFFF800000000000

Conversion rule:

    virtual = physical + HHDM_BASE

This allows direct access to physical memory through higher-half virtual addresses.

Used for:

    Frame allocators
    Page tables
    Framebuffer access
    Memory management
    Hardware mappings

---

## 11. Paging Activation

The bootloader:

    Builds page tables
    Builds identity mappings
    Builds HHDM mappings
    Loads CR3

before transferring control to the kernel.

As a result, the kernel begins execution with a functioning virtual memory environment.

---

## 12. Kernel Handoff

Before entering the kernel:

    Stack is initialized
    Paging is active
    Identity mapping exists
    HHDM exists
    BootInfo is populated

The bootloader installs a dedicated kernel stack before jumping to the kernel entry point.

Transfer interface:

    kernel_entry(&mut BootInfo) -> !

Execution never returns to the bootloader.

---

## 13. Current Responsibilities Summary

The modern Bulldog bootloader is responsible for:

    UEFI initialization
    GOP initialization
    Framebuffer setup
    Console output
    Kernel ELF loading
    ELF relocation
    FAT image loading
    Memory map conversion
    BootInfo construction
    Identity paging
    HHDM paging
    CR3 installation
    Kernel handoff

---

## 14. Future Enhancements

Planned future work:

    ACPI table handoff
    RSDP discovery
    IOAPIC support
    SMP processor startup
    Kernel command-line support
    KASLR
    Enhanced boot diagnostics

Not currently implemented:

    ACPI handoff
    SMP startup
    KASLR

---

## Contributor Notes

- BootInfo is the official bootloader-to-kernel contract.
- HHDM base must remain synchronized between bootloader and kernel.
- Changes to paging layout must be reflected in documentation.
- Any additions to BootInfo require updates in bootloader, boot-proto, and kernel.
- FAT image loading is part of the current boot architecture and must remain documented.

---

## License

MIT or Apache 2.0 — TBD.