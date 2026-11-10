## Bulldog Kernel – Memory Map (Physical & Virtual)

Reviewed: November 2026

This document describes Bulldog's current physical and virtual memory layout.

It reflects the implementation found in:

    bootloader/src/paging.rs
    boot-proto/src/lib.rs
    kernel/src/paging.rs
    kernel/src/memory.rs

This document reflects the current UEFI 0.41 architecture.

---

## 1. Overview

Bulldog uses a higher-half kernel architecture.

The bootloader establishes:

    Identity mappings
    Higher-Half Direct Map (HHDM)
    Initial page tables

before control is transferred to the kernel.

The kernel then inherits this virtual-memory environment and extends it as required.

---

## 2. Physical Memory Map

The physical memory map is provided by the bootloader through:

    BootInfo

Memory regions are represented using:

    MemoryRegion

Current region classifications include:

    Usable
    Reserved
    Acpi
    Mmio

The kernel uses this information to construct:

    Frame allocators
    Heap allocators
    Memory-management structures

---

## 3. Physical Memory Regions

Typical physical regions include:

### Usable RAM

Purpose:

    Page allocation
    Heap allocation
    User-memory allocation

---

### Reserved Memory

Purpose:

    Kernel image
    Firmware structures
    Protected regions

---

### ACPI Regions

Purpose:

    ACPI tables
    Firmware information

Current use:

    Reserved only

---

### MMIO Regions

Purpose:

    Hardware devices
    APIC registers
    Future drivers

These regions are never allocated as normal RAM.

---

## 4. Kernel Physical Image

The bootloader loads:

    kernel.elf

into physical memory before kernel startup.

BootInfo records:

    kernel_phys_start
    kernel_phys_end
    kernel_entry_phys

These values describe the executable kernel image.

---

## 5. Virtual Address Layout

Current conceptual layout:

    0xFFFFFFFFFFFFFFFF
    ────────────────────────────
        Higher-half kernel
        Kernel heap
        Kernel stacks
        HHDM
        MMIO mappings
    ────────────────────────────
    0xFFFF800000000000
        HHDM Base
    ────────────────────────────
        User mappings
    ────────────────────────────
    0x0000000000000000

This layout reflects the current bootloader-created address-space model.

---

## 6. Higher-Half Kernel Region

The higher-half region contains:

    Kernel text
    Kernel data
    Kernel heap
    Kernel stacks
    Runtime kernel structures

Kernel memory is not marked:

    USER_ACCESSIBLE

and therefore cannot be accessed from Ring 3.

---

## 7. Higher-Half Direct Map (HHDM)

Bulldog currently uses:

    HHDM_BASE
    0xFFFF800000000000

Address translation:

    virtual = physical + HHDM_BASE

The HHDM provides direct access to:

    Physical memory
    Page tables
    Frame allocator structures
    ELF loading buffers

The HHDM is created by the bootloader and inherited by the kernel.

---

## 8. Framebuffer Mapping

The bootloader provides:

    Framebuffer physical address

During paging initialization Bulldog computes:

    framebuffer_virt

using the physical-memory offset.

This allows the framebuffer to be accessed through virtual memory.

Current consumers include:

    Framebuffer subsystem
    Writer subsystem
    Logger subsystem

---

## 9. LAPIC Mapping

The LAPIC is mapped into kernel address space.

Current mapping:

    Physical:
        0xFEE00000

    Virtual:
        LAPIC_VIRT_BASE

Flags:

    PRESENT
    WRITABLE
    NO_EXECUTE

This mapping exists before APIC initialization begins.

---

## 10. User Memory Layout

Bulldog currently supports user-mode execution.

Current user allocations include:

    ELF segments
    User stacks

Examples:

    /bin/hello
    /bin/goodbye

Current user pages exist inside the active address space and are marked:

    USER_ACCESSIBLE

Separate per-process page tables are not yet implemented.

---

## 11. User Stack Layout

Current user stacks are created using:

    alloc_user_stack()

Characteristics:

    64 KiB
    4 KiB pages
    USER_ACCESSIBLE mappings

The stack-top address is passed to:

    enter_user_mode()

during process startup.

---

## 12. Current Kernel Mappings

Current kernel mappings include:

    Higher-half kernel
    HHDM
    LAPIC MMIO
    Framebuffer
    Heap
    GDT/TSS structures
    User pages

These mappings exist simultaneously within the active address space.

---

## 13. Current User Mappings

Current user mappings include:

    ELF text
    ELF data
    ELF BSS
    User stack

Pages are tracked through:

    USER_PAGES

to support cleanup and frame recycling.

---

## 14. Current Memory Lifecycle

Current boot flow:

    Bootloader discovers memory
            ↓
    BootInfo created
            ↓
    Kernel inherits mappings
            ↓
    Frame allocator created
            ↓
    Heap initialized
            ↓
    User memory available
            ↓
    Process executes
            ↓
    Memory reclaimed on exit

This reflects the current implementation rather than a planned design.

---

## 15. Current Limitations

Not currently implemented:

    Per-process page tables
    Guard pages
    Shared memory
    Demand paging
    Memory-mapped files
    Copy-on-write
    NUMA support

These features remain future work.

---

## 16. Future Enhancements

Planned future improvements:

    Per-process address spaces
    Scheduler-aware memory management
    Shared memory regions
    Copy-on-write support
    Memory-mapped files
    KASLR
    NUMA-aware allocation

These will build upon the current architecture.

---

## Contributor Notes

- HHDM assumptions must remain synchronized with the bootloader.
- BootInfo remains the authoritative source for memory layout information.
- User-mode support currently relies on USER_ACCESSIBLE mappings.
- Review this document whenever paging or HHDM behavior changes.
- Keep virtual address diagrams synchronized with implementation.

---

## License

MIT or Apache 2.0 — TBD.