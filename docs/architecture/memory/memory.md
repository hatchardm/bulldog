## Bulldog Kernel – Memory Management (Current Implementation)

Reviewed: November 2026

This document describes Bulldog's actual memory-management implementation as of the current kernel state.

It reflects the behavior of:

- memory.rs
- gdt.rs
- apic.rs
- kernel initialization
- ELF loading
- user-mode execution

This is not a future design document.

It describes the memory subsystem as it currently exists.

---

## 1. Overview

Bulldog currently implements:

- Higher-half kernel memory layout
- Higher-Half Direct Mapping (HHDM)
- Early physical frame allocation
- Bitmap-based frame tracking
- LAPIC MMIO mapping
- OffsetPageTable paging management
- ELF segment mapping
- User-page allocation
- User-stack allocation
- User-page tracking
- User-memory cleanup
- GDT/TSS with dedicated IST stacks

Advanced virtual-memory features remain future work.

---

## 2. Virtual Address Layout

Bulldog operates as a higher-half kernel.

The bootloader provides a physical-memory offset allowing physical memory to be accessed through virtual addresses.

Current conceptual layout:

    +------------------------------+
    | Higher-Half Kernel Space     |
    |------------------------------|
    | Kernel code                  |
    | Kernel data                  |
    | Kernel heap                  |
    | Kernel stacks                |
    | LAPIC MMIO mapping           |
    | Direct physical mapping      |
    +------------------------------+
    | Reserved                     |
    +------------------------------+
    | User-space mappings          |
    +------------------------------+
    | Lower virtual addresses      |
    +------------------------------+

---

## 3. Higher-Half Direct Mapping (HHDM)

Bulldog uses a higher-half direct map supplied by the bootloader.

Address conversion:

    virtual = physical + HHDM_OFFSET

The HHDM is used for:

    Page-table access
    Physical-frame access
    Frame allocator access
    Kernel memory management
    ELF segment loading

This allows the kernel to manipulate physical memory using normal virtual addresses.

---

## 4. Paging Infrastructure

Bulldog uses the standard x86_64 four-level paging architecture.

Current paging support includes:

    PML4
    PDPT
    PD
    PT

Current helper functions:

    active_level_4_table()
    init_offset_page_table()
    map_page()
    map_lapic_mmio()

The paging subsystem currently uses standard 4 KiB pages for dynamic mappings.

---

## 5. Active Page Table Access

The active page table is obtained by:

    Reading CR3
    Converting the physical address through HHDM
    Producing a mutable PageTable reference

This allows Bulldog to manipulate the currently active address space directly.

---

## 6. OffsetPageTable

Bulldog wraps page-table access using:

    OffsetPageTable

The offset page table uses the bootloader-provided physical-memory offset.

This abstraction forms the basis for:

    Virtual mappings
    ELF segment loading
    User-page allocation
    LAPIC mapping

---

## 7. LAPIC MMIO Mapping

The Local APIC is mapped into higher-half space.

Virtual address:

    LAPIC_VIRT_BASE

Physical address:

    0xFEE00000

Current flags:

    PRESENT
    WRITABLE
    NO_EXECUTE

The mapping is required before:

    setup_apic()

is executed.

Current LAPIC usage includes:

    APIC ID access
    Version reporting
    Timer configuration
    End-of-interrupt operations

---

## 8. Physical Memory Management

Bulldog currently uses two allocators.

### 8.1 PreHeapAllocator

Used during very early initialization.

Characteristics:

    Fixed-size allocator
    512 frame capacity
    No heap dependency
    Deterministic behavior

Purpose:

    Bootstrap memory allocation before heap initialization

---

### 8.2 BootInfoFrameAllocator

Used after normal runtime initialization.

Current capabilities:

    Dynamic frame storage
    Bitmap tracking
    Frame allocation
    Used-frame tracking
    Frame reuse support

This allocator becomes the primary frame allocator after startup.

---

## 9. Frame Bitmap

Bulldog currently tracks frame usage through:

    FrameBitmap

Current configuration:

    32768 bytes
    262144 tracked frames
    1 GiB coverage
    Base address: 0x100000

Capabilities:

    mark_used()
    mark_free()
    is_used()
    contains()

Frames currently support both allocation and reuse.

---

## 10. ELF Memory Mapping

Bulldog loads ELF programs directly into the current address space.

Current workflow:

    Validate ELF
            ↓
    Read program headers
            ↓
    Allocate frames
            ↓
    Create user mappings
            ↓
    Copy file contents
            ↓
    Zero BSS sections
            ↓
    Update page permissions

Supported ELF features:

    Loadable segments
    Read-only segments
    Writable segments
    Executable segments
    BSS initialization

---

## 11. User Memory Management

User-mode memory is currently implemented.

Current support includes:

    User page allocation
    User segment mapping
    User stacks
    User memory cleanup

This functionality enables:

    /bin/hello
    /bin/goodbye

and future user applications.

---

## 12. User Page Allocation

Bulldog allocates user memory through:

    map_user_page()

Current page flags:

    PRESENT
    WRITABLE
    USER_ACCESSIBLE

Every mapped page is recorded for later cleanup.

---

## 13. User Stack Allocation

User stacks are currently created by:

    alloc_user_stack()

Current implementation:

    64 KiB stack size
    4 KiB page granularity
    Higher user virtual addresses
    Automatic frame allocation

The top of the stack is returned to the ELF loader and later supplied to enter_user_mode().

---

## 14. User Page Tracking

Bulldog tracks pages belonging to the currently executing user program.

Current implementation:

    USER_PAGES

tracked as:

    Virtual page addresses

Pages are recorded whenever:

    User segments are mapped
    User stack pages are mapped

Purpose:

    Cleanup during sys_exit()
    Frame reclamation
    Process isolation

---

## 15. User Memory Cleanup

Process cleanup is currently implemented.

Cleanup occurs through:

    unmap_current_user()

Current cleanup sequence:

    Iterate tracked pages
    Unmap virtual pages
    Flush TLB entries
    Return physical frames
    Clear page tracker

This mechanism enables:

    hello
            ↓
    exit
            ↓
    goodbye
            ↓
    exit

without memory leaks.

---

## 16. User Access Enforcement

Bulldog provides:

    force_user_access()

This helper ensures:

    USER_ACCESSIBLE flags
    Executable permissions
    Correct privilege transitions

are configured throughout the active page-table hierarchy.

This function is used before entering Ring 3.

---

## 17. Interrupt Stacks and TSS

Bulldog currently maintains dedicated IST stacks.

### IST0

Purpose:

    Double-fault handling

### IST1

Purpose:

    LAPIC timer
    Page-fault handling

Characteristics:

    128 KiB each
    16-byte alignment
    Statically allocated
    Higher-half addresses

Guard pages are planned but not yet implemented.

---

## 18. Current Initialization Flow

Current startup memory flow:

    Bootloader provides memory map
            ↓
    OffsetPageTable created
            ↓
    PreHeapAllocator created
            ↓
    LAPIC MMIO mapped
            ↓
    GDT/TSS initialized
            ↓
    APIC initialized
            ↓
    Full frame allocator created
            ↓
    Heap initialized
            ↓
    User-mode memory available

This reflects the current implementation path.

---

## 19. Future Work

Not yet implemented:

    Guard pages
    Per-process address spaces
    Per-process page tables
    Copy-on-write
    Memory-mapped files
    Large-page support
    NUMA awareness
    Per-CPU frame allocators

These features will be revisited after scheduler development.

---

## Contributor Notes

- LAPIC mapping must occur before APIC initialization.
- HHDM assumptions must match bootloader behavior.
- User pages must always be tracked for cleanup.
- IST stacks must remain aligned and mapped.
- Process cleanup depends on correct page tracking.
- Memory-management changes should be reviewed against ELF loading and user-mode execution paths.

---

## License

MIT or Apache 2.0 — TBD.

