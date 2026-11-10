## Bulldog Kernel – Higher-Half Direct Map (HHDM)

Reviewed: November 2026

This document describes Bulldog's current Higher-Half Direct Map (HHDM) implementation.

The HHDM provides direct virtual access to physical memory through a fixed higher-half address offset.

The current implementation is established by the bootloader during early paging initialization and inherited by the kernel through BootInfo.

This document reflects the current UEFI 0.41 boot architecture.

---

## 1. Overview

The Higher-Half Direct Map (HHDM) is one of the core memory-management mechanisms used throughout Bulldog.

The HHDM provides:

    Direct access to physical memory
    Page-table access
    Frame allocator support
    Physical frame inspection
    ELF loading support
    User-memory management support

The HHDM eliminates the need to create temporary virtual mappings when accessing physical memory.

---

## 2. Mapping Formula

Bulldog currently uses:

    virtual = physical + HHDM_OFFSET

Current offset:

    0xFFFF800000000000

Examples:

    Physical: 0x0000000000100000
    Virtual : 0xFFFF800000100000

    Physical: 0x00000000FEE00000
    Virtual : 0xFFFF8000FEE00000

The offset is supplied by the bootloader and provided to the kernel through BootInfo.

---

## 3. Current Layout

Conceptually the address space resembles:

    0xFFFFFFFFFFFFFFFF
    ────────────────────────────
        Higher-half kernel
        Kernel heap
        Kernel stacks
        Direct physical map
        MMIO mappings
    ────────────────────────────
    0xFFFF800000000000
        HHDM Base
    ────────────────────────────
        Lower-half addresses
        User mappings
    ────────────────────────────
    0x0000000000000000

The HHDM provides contiguous access to physical memory through higher-half virtual addresses.

---

## 4. HHDM Creation

The HHDM is created by the bootloader.

Current sequence:

    Retrieve memory map
            ↓
    Create page tables
            ↓
    Create identity mapping
            ↓
    Create HHDM mapping
            ↓
    Load CR3
            ↓
    Transfer control to kernel

The kernel does not construct the HHDM.

The kernel inherits a fully operational HHDM environment from the bootloader.

---

## 5. BootInfo Integration

The bootloader passes:

    physical_memory_offset

through BootInfo.

This value allows the kernel to perform:

    Physical to virtual conversion
    Page-table access
    Physical frame access

BootInfo therefore acts as the authoritative source for HHDM configuration.

---

## 6. Page-Table Access

One of the most important HHDM uses is page-table access.

Current workflow:

    Read CR3
            ↓
    Obtain physical page-table address
            ↓
    Add HHDM offset
            ↓
    Produce valid virtual pointer

This mechanism is used by:

    active_level_4_table()
    init_offset_page_table()

Without the HHDM, the kernel would require additional temporary mappings for page-table manipulation.

---

## 7. OffsetPageTable Integration

Bulldog uses:

    OffsetPageTable

for virtual-memory management.

The OffsetPageTable relies directly on:

    physical_memory_offset

provided through BootInfo.

Current users include:

    map_page()
    map_lapic_mmio()
    map_user_page()
    map_user_segment()

The HHDM therefore underpins all current dynamic memory mappings.

---

## 8. Frame Allocator Integration

Frame allocation relies heavily on HHDM access.

Current activities include:

    Frame discovery
    Bitmap management
    Frame allocation
    Frame reuse

The frame allocator accesses physical structures through higher-half virtual addresses derived from the HHDM.

---

## 9. ELF Loader Integration

The ELF loader currently uses the HHDM during program loading.

Current workflow:

    Allocate physical frame
            ↓
    Map user page
            ↓
    Translate virtual address
            ↓
    Obtain physical address
            ↓
    Access physical memory through HHDM
            ↓
    Copy ELF data

The HHDM therefore provides the bridge between physical frames and ELF segment initialization.

---

## 10. User Memory Support

The HHDM is also used by Bulldog's user-memory subsystem.

Current support includes:

    User page allocation
    User stack allocation
    ELF segment mapping
    BSS initialization

During user-program loading, data is copied directly into physical pages using HHDM addresses.

This eliminates the need for temporary kernel mappings.

---

## 11. User Memory Cleanup

The process subsystem uses:

    unmap_current_user()

to clean up memory after process termination.

Cleanup includes:

    Unmapping pages
    Releasing frames
    Returning frames to the allocator

The HHDM makes frame management and page-table manipulation straightforward during cleanup operations.

---

## 12. Current Invariants

The following assumptions must remain true:

    HHDM offset must remain constant after boot
    HHDM must exist before kernel execution begins
    HHDM must map physical memory consistently
    BootInfo offset must match bootloader configuration
    Page-table access must remain HHDM based

Violating these assumptions will destabilize the memory subsystem.

---

## 13. Current Uses

The HHDM is currently used by:

    Paging subsystem
    Page-table management
    Frame allocator
    LAPIC mapping
    ELF loading
    User memory
    Process cleanup

Almost all low-level memory-management operations depend on it.

---

## 14. Future Enhancements

Possible future work:

    KASLR-aware HHDM
    Larger HHDM coverage
    NUMA-aware memory regions
    Per-CPU memory optimizations
    Additional MMIO integration

These features are not currently implemented.

---

## Contributor Notes

- The bootloader owns HHDM creation.
- The kernel owns HHDM usage.
- physical_memory_offset is the canonical HHDM reference.
- Review HHDM behaviour whenever bootloader paging changes.
- Page-table helpers depend directly on correct HHDM operation.
- User-mode memory management relies on HHDM-backed frame access.

---

## License

MIT or Apache 2.0 — TBD.