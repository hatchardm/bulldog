## Bulldog Kernel – Memory Layout (Current Implementation)

Reviewed: November 2026

Disclaimer:

This document describes Bulldog's current memory layout as implemented today.

It does not represent the final long-term memory architecture.

Features such as:

    Per-process address spaces
    Guard pages
    Copy-on-write
    Memory-mapped files
    KASLR
    NUMA support

remain future work and will be documented when implemented.

For subsystem-specific details see:

    memory.md
    hhdm.md
    paging.md
    page-table.md
    frame-allocator.md
    kernel_init.md

This document provides a high-level view of Bulldog's current physical and virtual memory layout.

---

## 1. Overview

Bulldog currently uses a higher-half kernel architecture featuring:

    Higher-half kernel execution
    Higher-Half Direct Mapping (HHDM)
    Statically allocated kernel stacks
    Dedicated IST stacks
    Fixed LAPIC MMIO mappings
    User-mode memory support
    Shared address-space model

Current limitations include:

    No guard pages
    No per-process page tables
    No user heap
    No copy-on-write

The layout is intentionally simple and optimized for debugging and architectural clarity.

---

## 2. Virtual Address Space (Current)

    0xFFFFFFFFFFFFFFFF
    ────────────────────────────────
        Higher-Half Kernel Space

        Kernel text
        Kernel data
        Kernel heap
        Kernel stacks
        LAPIC MMIO mappings
        HHDM region
        Runtime kernel structures

    ────────────────────────────────
    0xFFFF800000000000
        HHDM Base

    ────────────────────────────────

        User mappings
        ELF segments
        User stacks

    ────────────────────────────────
    0x0000000000000000

### Key Points

- Kernel executes entirely in higher-half memory.
- HHDM supplies direct access to physical memory.
- LAPIC MMIO uses a fixed higher-half address.
- User-mode memory is implemented.
- User pages coexist within the current active address space.
- Separate process address spaces are future work.

---

## 3. Kernel Virtual Regions

### Kernel Text / Read-Only Data

Loaded by the bootloader and executed in higher-half memory.

Characteristics:

    Read-only
    Executable

Contains:

    Kernel code
    Constant data

---

### Kernel Data / BSS

Characteristics:

    Read/Write
    Non-executable

Contains:

    Global variables
    Runtime kernel state

---

### Kernel Heap

Initialized during:

    kernel_init()

Memory-management components include:

    PreHeapAllocator
    BulldogFrameAllocator
    FrameBitmap

The heap provides dynamic memory allocation for higher-level kernel subsystems.

---

### Kernel Stacks

Bulldog currently maintains:

    Main kernel stack
    Double-fault IST stack
    LAPIC/Page-fault IST stack

Current characteristics:

    128 KiB each
    16-byte aligned
    Statically allocated
    Higher-half addresses

Guard pages are not currently implemented.

---

## 4. Higher-Half Direct Map (HHDM)

Bulldog uses a bootloader-provided direct physical mapping.

Address conversion:

    virtual = physical + physical_memory_offset

The HHDM is used for:

    Page-table access
    Frame allocation
    ELF loading
    User-memory management
    Physical-memory inspection

The HHDM implementation is documented in:

    memory/hhdm.md

---

## 5. LAPIC MMIO Region

The Local APIC is mapped at a fixed virtual address.

Virtual:

    LAPIC_VIRT_BASE

Physical:

    0xFEE00000

Current flags:

    PRESENT
    WRITABLE
    NO_EXECUTE

Used by:

    lapic_read()
    lapic_write()
    setup_apic()

The mapping exists before APIC initialization begins.

---

## 6. Physical Memory Layout

The bootloader provides the physical-memory map through:

    BootInfo

Typical regions include:

    Low memory
    Kernel image
    Usable RAM
    Reserved regions
    ACPI regions
    MMIO regions

The kernel uses this information to construct:

    BulldogFrameAllocator
    FrameBitmap
    Heap allocator
    Memory-management structures

---

## 7. User Memory Layout

Bulldog currently supports user-mode memory.

Current allocations include:

    ELF text segments
    ELF data segments
    ELF BSS segments
    User stacks

Examples:

    /bin/hello
    /bin/goodbye

Current user pages are marked:

    USER_ACCESSIBLE

and tracked for cleanup and frame recycling.

---

## 8. User Stack Layout

User stacks are created through:

    alloc_user_stack()

Current implementation:

    64 KiB size
    4 KiB pages
    Dynamically allocated frames
    USER_ACCESSIBLE permissions

The stack top is passed directly to:

    enter_user_mode()

during process startup.

---

## 9. Current Kernel Mappings

Current kernel mappings include:

    Kernel image
    Kernel heap
    Kernel stacks
    HHDM
    LAPIC MMIO
    Framebuffer
    GDT
    TSS
    IST stacks

These mappings remain active throughout kernel execution.

---

## 10. Current User Mappings

Current user mappings include:

    ELF segments
    User stacks

User pages are tracked through:

    USER_PAGES

to support:

    Process cleanup
    Frame recycling
    Memory reclamation

---

## 11. Reserved Regions

The following areas are reserved for future growth:

### Future Kernel Regions

    Additional MMIO regions
    IOAPIC mappings
    Per-CPU data
    Scheduler structures