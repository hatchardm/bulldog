## Bulldog Kernel – Memory Subsystem Index

Reviewed: November 2026

The memory subsystem defines how Bulldog manages:

    Physical memory
    Virtual memory
    Paging structures
    Higher-Half Direct Mapping (HHDM)
    Frame allocation
    User-memory mappings
    ELF loading support

It is one of the core architecture subsystems and forms the foundation for:

    Paging
    Heap allocation
    ELF loading
    User-mode execution
    Process management
    Future scheduling

This directory contains the current memory architecture documentation.

---

## Contents

### 1. Memory Overview

    memory.md

High-level description of Bulldog's current memory-management implementation.

Topics include:

    Higher-half architecture
    HHDM
    Paging infrastructure
    Frame allocation
    ELF loading
    User memory
    User-page tracking
    Process cleanup

---

### 2. Memory Map

    memory-map.md

Describes the current physical and virtual memory layout.

Topics include:

    Physical memory regions
    BootInfo handoff
    Virtual address layout
    HHDM layout
    Kernel mappings
    User mappings
    Framebuffer mappings
    LAPIC mappings

---

### 3. Frame Allocator

    frame-allocator.md

Describes Bulldog's current frame-allocation implementation.

Topics include:

    BootInfo integration
    BulldogFrameAllocator
    Sequential allocation
    Frame recycling
    Free-list management
    User-memory cleanup integration
    ELF-loader integration

---

### 4. Higher-Half Direct Map (HHDM)

    hhdm.md

Describes Bulldog's direct physical-memory mapping architecture.

Topics include:

    HHDM offset
    Physical-to-virtual conversion
    Bootloader integration
    BootInfo integration
    OffsetPageTable usage
    ELF loading support
    User-memory support

---

### 5. Paging Architecture

    paging.md

Describes Bulldog's runtime paging implementation.

Topics include:

    OffsetPageTable
    Paging initialization
    BulldogFrameAllocator
    Page mappings
    User-page allocation
    User-stack allocation
    Process cleanup
    Page permissions

---

### 6. Page Table Structures

    page-table.md

Describes the x86_64 paging hierarchy used by Bulldog.

Topics include:

    PML4
    PDPT
    PD
    PT
    Page-table flags
    USER_ACCESSIBLE mappings
    NO_EXECUTE support
    Ring 3 memory access

---

## Current Architecture

Current memory-management flow:

    Bootloader
            ↓
    Memory Map Discovery
            ↓
    BootInfo Handoff
            ↓
    HHDM Initialization
            ↓
    Paging Initialization
            ↓
    Frame Allocator Creation
            ↓
    Heap Initialization
            ↓
    ELF Loading
            ↓
    User Memory
            ↓
    Process Execution
            ↓
    Process Cleanup

This reflects the current Bulldog implementation.

---

## Responsibilities

The memory subsystem currently provides:

    Higher-half execution
    HHDM access
    Frame allocation
    Paging management
    User-page allocation
    User-stack allocation
    ELF segment mapping
    Page cleanup
    Frame recycling

These capabilities support all current user-mode and process-management features.

---

## Related Documentation

### Boot Layer

    ../../boot/bootloader.md
    ../../boot/boot.md

Provides:

    BootInfo
    Memory-map handoff
    HHDM creation
    Paging bootstrap

---

### Architecture Layer

    ../kernel_init.md
    ../kernel-bin.md
    ../cpu-init.md

Provides:

    Kernel startup
    Paging initialization
    APIC mappings
    Runtime initialization

---

### Process Layer

    ../processes.md

Documents:

    User-mode execution
    Process lifecycle
    Process cleanup
    Scheduler planning

---

## Current Status

Implemented:

    Higher-half kernel
    HHDM
    Paging
    Frame allocation
    Frame recycling
    User-memory mappings
    User stacks
    ELF loading
    User-page tracking
    Process cleanup

Future Work:

    Per-process page tables
    Copy-on-write
    Demand paging
    Shared memory
    Memory-mapped files
    NUMA support

---

## Contributor Notes

- Keep memory documentation synchronized with BootInfo changes.
- Review memory documentation whenever paging changes.
- Synchronize HHDM documentation with bootloader paging.
- Update this index whenever new memory documents are added.
- Maintain consistency between memory, paging, and process documentation.

---

## License

MIT or Apache 2.0 — TBD.