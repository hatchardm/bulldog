## Bulldog Kernel – Paging Architecture

Reviewed: November 2026

This document describes Bulldog's current paging architecture.

It reflects the implementation found in:

    kernel/src/paging.rs
    kernel/src/memory.rs
    bootloader/src/paging.rs
    boot-proto/src/lib.rs

This document describes the current implementation rather than future design goals.

---

## 1. Overview

Bulldog operates using x86_64 four-level paging.

Current architecture:

    Bootloader creates page tables
            ↓
    Bootloader creates HHDM
            ↓
    Bootloader enables paging
            ↓
    Bootloader loads CR3
            ↓
    Control transfers to kernel
            ↓
    Kernel attaches to existing mappings

The kernel does not build the initial page-table hierarchy.

The kernel inherits a functioning virtual-memory environment from the bootloader.

---

## 2. Paging Model

Bulldog currently uses:

    64-bit Long Mode
    4-Level Paging

Paging hierarchy:

    PML4
      ↓
    PDPT
      ↓
    PD
      ↓
    PT
      ↓
    4 KiB Pages

The kernel primarily operates using 4 KiB mappings for dynamic allocations.

The bootloader uses larger mappings during early startup.

---

## 3. Relationship To The Bootloader

The bootloader is responsible for:

    Creating page tables
    Creating identity mappings
    Creating HHDM mappings
    Loading CR3
    Entering the kernel

The kernel is responsible for:

    Attaching to existing page tables
    Extending mappings
    User-mode mappings
    MMIO mappings
    Process memory mappings

This keeps paging bootstrap logic separate from normal runtime paging.

---

## 4. Physical Memory Offset

The bootloader provides:

    physical_memory_offset

through BootInfo.

This value defines the Higher-Half Direct Map.

Current conversion:

    virtual = physical + physical_memory_offset

The physical-memory offset is used throughout the paging subsystem.

---

## 5. OffsetPageTable

Bulldog uses:

    OffsetPageTable

as its primary paging abstraction.

During initialization:

    Read CR3
            ↓
    Convert physical address through HHDM
            ↓
    Obtain PML4
            ↓
    Construct OffsetPageTable

This object becomes the foundation for all later mappings.

---

## 6. paging::init()

Paging initialization occurs during early kernel startup.

Current responsibilities:

    Obtain physical memory offset
    Create frame allocator
    Attach OffsetPageTable
    Establish global mapper access
    Expose framebuffer virtual address

Current output:

    MAPPER
    FRAME_ALLOCATOR

These global objects are later used throughout the kernel.

---

## 7. BulldogFrameAllocator

The paging subsystem includes:

    BulldogFrameAllocator

Current responsibilities:

    Frame allocation
    Frame reuse
    Frame recycling

Allocator state:

    next frame pointer
    usable memory limit
    free-list storage

Current allocation order:

    Recycled frame
            ↓
    Free-list empty
            ↓
    Sequential physical allocation

---

## 8. Frame Reuse

Bulldog currently supports frame recycling.

Current workflow:

    Process exits
            ↓
    User pages unmapped
            ↓
    Physical frames returned
            ↓
    Frames added to free list

Future allocations may reuse these frames.

This has been verified through:

    hello
            ↓
    goodbye

process chaining tests.

---

## 9. Mapping Pages

Bulldog currently provides:

    map_page()

A mapping operation consists of:

    Virtual address
    Physical address
    Page flags
    Frame allocator

Flags currently used include:

    PRESENT
    WRITABLE
    USER_ACCESSIBLE
    NO_EXECUTE

Existing mappings may have their flags updated rather than recreated.

---

## 10. LAPIC MMIO Mapping

Bulldog maps the Local APIC into kernel virtual space.

Current mapping:

    Physical:
        0xFEE00000

    Virtual:
        LAPIC_VIRT_BASE

Current flags:

    PRESENT
    WRITABLE
    NO_EXECUTE

The mapping is created before:

    setup_apic()

is executed.

---

## 11. User Memory Mapping

Bulldog currently supports user-memory mappings.

Current functionality:

    map_user_page()
    map_user_segment()

Used for:

    ELF loading
    Program execution
    User stacks

All user mappings are created with:

    USER_ACCESSIBLE

permissions.

---

## 12. ELF Segment Mapping

User programs are loaded through:

    map_elf_segment()

Current responsibilities:

    Allocate frames
    Create mappings
    Apply permissions
    Copy segment data
    Zero BSS regions

Supported segment types:

    Read-only
    Writable
    Executable

Page permissions are adjusted after loading completes.

---

## 13. User Stack Allocation

Bulldog currently supports user stacks.

Current implementation:

    alloc_user_stack()

Characteristics:

    64 KiB stack
    4 KiB pages
    User-accessible mappings
    Dynamically allocated frames

The resulting stack-top address is supplied to:

    enter_user_mode()

during process startup.

---

## 14. User Page Tracking

All user pages are tracked.

Current structure:

    USER_PAGES

Purpose:

    Process cleanup
    Memory reclamation
    Frame reuse

Tracked objects include:

    ELF segment pages
    User stack pages

---

## 15. Process Cleanup

User memory is reclaimed through:

    unmap_current_user()

Current cleanup sequence:

    Locate tracked pages
            ↓
    Unmap page
            ↓
    Flush TLB
            ↓
    Return frame
            ↓
    Add frame to free list

This permits repeated user-mode execution without exhausting memory.

---

## 16. Page Permissions

Current permissions include:

    PRESENT
    WRITABLE
    USER_ACCESSIBLE
    NO_EXECUTE

Read-only ELF segments become:

    PRESENT
    USER_ACCESSIBLE

after loading is complete.

This prevents modification of executable code and read-only data.

---

## 17. User Accessibility

Before entering Ring 3, Bulldog executes:

    force_user_access()

This function walks:

    PML4
    PDPT
    PD
    PT

and ensures:

    USER_ACCESSIBLE

permissions are present throughout the translation path.

This guarantees that user-mode code can access its own memory.

---

## 18. Page Fault Handling

Bulldog currently installs page-fault handlers.

Current capabilities include:

    Fault detection
    Diagnostic logging
    IST-based execution

Current implementation is diagnostic-oriented.

Demand paging is not currently implemented.

---

## 19. Current Invariants

The following assumptions must remain true:

    HHDM offset remains valid
    OffsetPageTable remains attached
    User pages are tracked
    User frames are reclaimable
    LAPIC mapping exists before APIC setup

Violating these assumptions will destabilize the paging subsystem.

---

## 20. Future Enhancements

Planned future work:

    Per-process page tables
    Copy-on-write
    Guard pages
    Demand paging
    Shared memory
    Large-page support
    NUMA awareness
    Per-CPU address-space optimizations

These features are not currently implemented.

---

## Contributor Notes

- Bootloader owns initial paging construction.
- Kernel owns paging management after handoff.
- OffsetPageTable is the preferred interface for mappings.
- User pages must always be tracked for cleanup.
- Frame recycling is now part of the paging lifecycle.
- Review paging assumptions whenever bootloader paging changes.

---

## License

MIT or Apache 2.0 — TBD.