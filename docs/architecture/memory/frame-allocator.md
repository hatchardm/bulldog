## Bulldog Kernel – Frame Allocator

Reviewed: November 2026

This document describes Bulldog's current physical-frame allocator implementation.

The frame allocator is responsible for supplying physical memory frames used by:

    Paging
    User-memory mappings
    ELF loading
    Kernel allocations
    MMIO mappings

It is one of the lowest-level memory-management components in the system.

This document reflects the current implementation found in:

    kernel/src/paging.rs
    kernel/src/memory.rs

---

## 1. Overview

Bulldog currently implements a simple frame allocator designed for reliability and observability during OS bring-up.

Current features:

    BootInfo-driven initialization
    Sequential frame allocation
    Free-list recycling
    User-frame reclamation
    Frame reuse

The allocator currently favors simplicity over advanced allocation strategies.

---

## 2. BootInfo Integration

The allocator is constructed using information provided by:

    BootInfo

Input sources:

    Memory regions
    Memory-region types
    Framebuffer information

The allocator uses only regions marked:

    MemoryRegionKind::Usable

All other region types are ignored.

---

## 3. Region Selection

During initialization Bulldog scans the memory map looking for a usable region.

Acceptance criteria:

    Region is marked usable
    Region is at least 2 MiB
    Region does not overlap the framebuffer

The first acceptable region becomes the allocator's source of frames.

Current diagnostics report:

    Allocator start address
    Allocator end address

during startup.

---

## 4. BulldogFrameAllocator

Bulldog currently uses:

    BulldogFrameAllocator

Internal state:

    next
    end
    free_list

Meaning:

    next
        Next sequential frame

    end
        End of usable region

    free_list
        Recycled frames

This structure is initialized during:

    paging::init()

and remains globally available thereafter.

---

## 5. Sequential Allocation

When no recycled frames exist, allocation is performed linearly.

Current algorithm:

    frame = next
            ↓
    next += 4096
            ↓
    return frame

Frames are allocated in 4 KiB increments.

Allocation fails when:

    next + 4096 > end

This behavior is deterministic and easy to debug.

---

## 6. Frame Recycling

Bulldog now supports frame reuse.

Current workflow:

    User process exits
            ↓
    User pages unmapped
            ↓
    Frames returned
            ↓
    Frames stored in free_list

This capability was introduced alongside the user-mode and process-management subsystems.

---

## 7. Allocation Order

Current priority:

    free_list
            ↓
    sequential allocation

Allocation process:

    Check free_list
            ↓
    Frame available?
            ↓
    Yes → reuse frame
            ↓
    No
            ↓
    Allocate from next

This prevents unnecessary consumption of new physical memory.

---

## 8. Frame Release

Frames are returned through:

    free_frame()

Current users:

    unmap_current_user()

Frame release occurs when user-memory cleanup is performed during:

    sys_exit()

This allows memory to be recycled for subsequent user processes.

---

## 9. User-Mode Integration

The frame allocator is now heavily involved in user-mode execution.

Current responsibilities include:

    User segment allocation
    User-stack allocation
    ELF loading
    Process cleanup

Current helpers using the allocator:

    map_user_page()
    map_user_segment()
    alloc_user_stack()

---

## 10. ELF Loader Integration

The ELF loader allocates frames through the frame allocator.

Current workflow:

    Parse ELF
            ↓
    Read program headers
            ↓
    Allocate physical frames
            ↓
    Create mappings
            ↓
    Copy ELF segment data
            ↓
    Zero BSS

The allocator provides the physical memory backing all executable user programs.

---

## 11. Paging Integration

The frame allocator supplies frames to:

    Page-table creation
    User-page mappings
    MMIO mappings
    Runtime virtual-memory management

Common users include:

    map_page()
    map_lapic_mmio()
    map_user_page()

The paging subsystem depends directly on allocator correctness.

---

## 12. Relationship To HHDM

Allocated frames are accessed through:

    Higher-Half Direct Map (HHDM)

Current usage:

    Frame inspection
    ELF segment copying
    Memory initialization

This allows physical memory to be manipulated through virtual addresses.

---

## 13. Current Limitations

Current limitations include:

    Single usable-region selection
    No region coalescing
    No allocation statistics
    No NUMA awareness
    No guard frames
    No allocation balancing

These limitations are acceptable during the current stage of development.

---

## 14. Safety Invariants

The allocator must:

    Allocate only usable RAM
    Avoid framebuffer overlap
    Avoid MMIO regions
    Avoid reserved firmware regions
    Avoid ACPI regions
    Return only aligned frames

Violation of these assumptions will corrupt memory management.

---

## 15. Relationship To FrameBitmap

Bulldog also contains:

    FrameBitmap

The bitmap subsystem currently provides:

    mark_used()
    mark_free()
    is_used()

The allocator currently relies primarily on:

    Sequential allocation
    free_list recycling

Future revisions may integrate allocator and bitmap management more tightly.

---

## 16. Current Lifecycle

Current allocation lifecycle:

    BootInfo
            ↓
    BulldogFrameAllocator
            ↓
    allocate_frame()
            ↓
    User/Kernel Mapping
            ↓
    Process Exit
            ↓
    free_frame()
            ↓
    free_list
            ↓
    Reuse

This lifecycle has been verified through:

    hello
            ↓
    goodbye

user-process chaining tests.

---

## 17. Future Enhancements

Planned improvements:

    Full bitmap allocation
    Multi-region support
    Guard frames
    Allocation statistics
    NUMA awareness
    Per-CPU frame pools
    Fragmentation analysis

These features are not currently implemented.

---

## Contributor Notes

- Keep allocator behavior deterministic.
- Preserve allocator diagnostics during development.
- User-memory cleanup depends on frame recycling.
- Frame reuse is intentional and should not be removed.
- Review allocator behavior whenever paging or user-memory code changes.
- Avoid introducing complexity before scheduler work stabilizes.

---

## License

MIT or Apache 2.0 — TBD.