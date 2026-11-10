## Bulldog Kernel – Allocator Subsystem Overview

Reviewed: November 2026

Bulldog's allocator subsystem provides the kernel heap used by all dynamic memory allocations.

Examples include:

    Box
    Vec
    String
    Collections
    VFS structures
    Process records
    Filesystem metadata

The allocator is a hybrid design combining:

    Fixed-size block allocation
    Linked-list fallback allocation

The allocator subsystem is initialized during:

    kernel_init()

after paging and frame allocation are available.

---

## Current Status

Implemented:

    Hybrid heap allocator
    Fixed-size free lists
    Linked-list fallback allocator
    2 MiB kernel heap
    Global allocator integration
    Dynamic allocation support

Future Work:

    User-mode heap support
    Guard pages
    Slab allocators
    Per-CPU heaps
    NUMA-aware allocation

---

## Components

### Fixed-Size Block Allocator

Located in:

    kernel/src/allocator/fixed_size_block.rs

Provides O(1) allocation for common block sizes:

    8
    16
    32
    64
    128
    256
    512
    1024
    2048
    4096

bytes.

Each size class maintains its own free-list.

Characteristics:

    Deterministic
    Fast allocation
    Fast deallocation
    No fragmentation within size classes

Suitable for:

    Kernel objects
    Metadata structures
    Small buffers
    Internal control structures

---

### Linked-List Fallback Allocator

Located in:

    kernel/src/allocator/linked_list.rs

Backed by:

    linked_list_allocator::LockedHeap

Used when:

    No fixed-size block fits
    Free-list is empty
    Allocation exceeds 4096 bytes

Characteristics:

    General-purpose allocation
    In-place metadata
    Region splitting
    Deterministic behaviour

Provides the fallback path for unusual or large allocations.

---

### Allocator Wiring

Located in:

    kernel/src/allocator.rs

Responsible for:

    Mapping heap pages
    Aligning heap start
    Splitting allocator regions
    Initializing both allocators
    Exposing the global allocator

The kernel allocator is exposed through:

    #[global_allocator]
    static ALLOCATOR

All kernel allocations are routed through this allocator automatically.

---

## Heap Layout

Current configuration:

    HEAP_START = 0x4444_4444_0000
    HEAP_SIZE  = 2 MiB

The heap is split into:

    Fixed-size region
    Fallback region

Current strategy:

    First half  -> fixed-size blocks
    Second half -> fallback allocator

This separation prevents overlap between allocator tiers.

---

## Initialization Flow

Current startup sequence:

    Map heap pages
            ↓
    Align heap start
            ↓
    Split heap region
            ↓
    Initialize fallback allocator
            ↓
    Populate fixed-size free-lists
            ↓
    Expose global allocator
            ↓
    Heap ready

After initialization:

    Box
    Vec
    String
    Collections

can be used by kernel subsystems.

---

## Integration With Kernel Initialization

The allocator is initialized during:

    init_heap()

This occurs after:

    Paging initialization
    HHDM availability
    Frame allocator initialization

and before:

    FAT32 initialization
    VFS initialization
    User-mode execution
    Process loading

This ensures all higher-level subsystems have a working heap.

---

## Current Architecture

Current allocation path:

    Allocation Request
            ↓
    Determine Size Class
            ↓
    Fixed-size Free-list
            ↓
       Available?
        /      \
      Yes      No
       |        |
       ↓        ↓
    Return   Fallback
     Block   Allocator
       ↓        ↓
       └────────┘
            ↓
        Allocation

Current deallocation path:

    Free Memory
            ↓
    Determine Size Class
            ↓
    Return To Free-list
         or
    Return To Fallback Heap

---

## Design Goals

The allocator is designed to provide:

    Deterministic behaviour
    Low fragmentation
    Fast common allocations
    Clear subsystem boundaries
    Simplicity
    Reliability

The design intentionally favors predictability over maximum allocation performance.

---

## Future Extensions

Short-Term:

    Allocation statistics
    Guard pages
    Debug allocation tracking

Mid-Term:

    Slab allocator integration
    Per-CPU heaps
    Kernel object caches

Long-Term:

    User-mode heap support
    Memory-mapped file buffers
    NUMA-aware allocation
    Advanced allocation policies

---

## Related Documentation

    allocator.md
    design.md
    fixed-size-block.md
    linked-list.md

Related architecture documents:

    ../memory/memory.md
    ../memory/frame-allocator.md
    ../kernel_init.md

---

## Contributor Notes

- Heap pages must be mapped before allocator initialization.
- Fixed-size and fallback regions must remain non-overlapping.
- Allocation metadata is stored inside allocator regions.
- Global allocator changes should be reviewed alongside allocator.md and design.md.
- Keep allocator behaviour deterministic and observable.

---

## License

MIT or Apache 2.0 — TBD.