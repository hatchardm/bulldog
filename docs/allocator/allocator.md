## Bulldog Kernel – Heap Allocator (allocator.rs)

Reviewed: November 2026

This document describes the top-level kernel heap allocator defined in:

    kernel/src/allocator.rs

The allocator wires together:

    FixedSizeBlockAllocator
    Linked-list fallback allocator

It maps the kernel heap, initializes allocator regions, and exposes the global allocator used by all kernel allocations.

The allocator is syscall-agnostic and currently serves kernel subsystems only.

---

## 1. Overview

Bulldog uses a hybrid heap allocator.

Current architecture:

    Fixed-size allocator
            ↓
    Fallback allocator

This design provides:

    Fast common allocations
    Deterministic behaviour
    Low fragmentation
    General-purpose heap support

The allocator is initialized during:

    kernel_init()

after paging and frame allocation are available.

---

## 2. Heap Layout

Current heap configuration:

    HEAP_START = 0x4444_4444_0000
    HEAP_SIZE  = 2 MiB

The heap is mapped during kernel initialization using the active page tables.

After alignment, the heap is divided into:

    Fixed-size region
    Fallback region

Current strategy:

    First half  → Fixed-size allocator
    Second half → Fallback allocator

This guarantees the two allocators never overlap.

---

## 3. Heap Initialization (init_heap)

The allocator is initialized through:

    init_heap()

Current responsibilities:

    Map heap pages
    Align heap start
    Split allocator regions
    Initialize fallback allocator
    Populate fixed-size free-lists
    Activate global allocator

This process occurs exactly once during startup.

---

## 4. Page Mapping

Each page in the heap range is mapped using:

    Mapper<Size4KiB>
    FrameAllocator<Size4KiB>

Page flags:

    PRESENT
    WRITABLE

Each heap page receives a backing physical frame from the kernel frame allocator.

Allocation fails if frames cannot be obtained.

---

## 5. Heap Alignment

Before allocator initialization:

    HEAP_START

is aligned upward using:

    align_up()

Current alignment:

    MAX_BLOCK_ALIGN = 1024

Purpose:

    Correct block alignment
    Free-list integrity
    Safe fixed-size allocations

The aligned address becomes the actual allocator start.

---

## 6. Region Splitting

After alignment the heap is divided into two independent regions.

Current layout:

    [ Fixed-Size Region ]
    [ Fallback Region   ]

Current allocation:

    50% fixed-size
    50% fallback

Current benefits:

    No region overlap
    Simple initialization
    Predictable behaviour
    Independent allocator state

Future versions may use dynamic sizing strategies.

---

## 7. Fixed-Size Region

The first half of the heap is reserved for:

    FixedSizeBlockAllocator

Supported block classes:

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

Characteristics:

    O(1) allocation
    O(1) deallocation
    Free-list based
    Low fragmentation

This allocator handles the majority of kernel allocations.

---

## 8. Fallback Region

The second half of the heap is reserved for:

    linked_list_allocator::LockedHeap

Current responsibilities:

    Large allocations
    Atypical layouts
    Fixed-size allocation overflow

The fallback allocator is initialized first before fixed-size block lists are populated.

Safety depends on non-overlapping allocator regions.

---

## 9. Allocator Initialization

After the heap is split:

    init_with_regions(
        fixed_start,
        fixed_size,
        fallback_start,
        fallback_size
    )

is called.

This performs:

    Fallback allocator initialization
    Fixed-size free-list population

Once complete the heap becomes available globally.

---

## 10. Global Allocator

Bulldog exposes:

    #[global_allocator]
    static ALLOCATOR

Current type:

    Locked<FixedSizeBlockAllocator>

This means:

    Box
    Vec
    String
    Collections
    Kernel subsystem allocations

all use the hybrid allocator automatically.

Subsystems do not need direct allocator references.

---

## 11. Thread Safety

Allocator access is synchronized through:

    spin::Mutex

wrapped by:

    Locked<T>

Current benefits:

    Safe shared access
    Simple locking model
    Interior mutability

This design is appropriate for the current single-core architecture.

Future SMP support may require allocator refinements.

---

## 12. Current Allocation Flow

Allocation request:

    Allocation Request
            ↓
    Determine Size Class
            ↓
    Free-list Available?
        /       \
      Yes       No
       |         |
       ↓         ↓
     Allocate  Fallback
       |      Allocator
       ↓         ↓
       └─────────┘
            ↓
        Allocation

This design provides fast common allocations while retaining general-purpose compatibility.

---

## 13. Current Deallocation Flow

Free request:

    Deallocation
            ↓
    Determine Size Class
        /       \
      Yes       No
       |         |
       ↓         ↓
    Return To  Return To
    Free-list  Fallback
                 Heap

This allows efficient reuse of small blocks.

---

## 14. Safety Invariants

The allocator depends on the following assumptions.

### Heap Pages Must Be Mapped

Initialization requires:

    Valid virtual mappings
    Writable pages

before allocator setup begins.

---

### Regions Must Not Overlap

Fixed-size and fallback regions must remain completely separate.

Violation causes allocator corruption.

---

### Alignment Must Be Preserved

Alignment guarantees:

    Block correctness
    Free-list consistency
    Metadata safety

Misalignment may corrupt allocator state.

---

### Initialization Occurs Once

The allocator must only be initialized a single time.

Reinitialization invalidates free-list state.

---

### Metadata Integrity

Allocator metadata resides inside allocator-managed memory.

Subsystems must never directly modify allocator structures.

---

## 15. Integration With Kernel Initialization

The allocator is initialized after:

    Paging initialization
    HHDM availability
    Frame allocator initialization

and before:

    FAT32 initialization
    VFS initialization
    User-mode execution
    Process creation

This ensures all high-level subsystems have dynamic memory available.

---

## 16. Current Limitations

Current limitations include:

    Single heap region
    Fixed 50/50 region split
    No heap growth
    No allocation statistics
    No guard pages
    No per-CPU heaps

These limitations are acceptable at the current stage of development.

---

## 17. Future Extensions

Short-Term:

    Allocation statistics
    Debug allocation tracking
    Guard pages

Mid-Term:

    Slab allocator integration
    Per-CPU heaps
    Kernel object caches

Long-Term:

    User-mode heap support
    NUMA-aware allocation
    Advanced allocation policies
    Dynamic heap expansion

---

## Related Documentation

    README.md
    design.md
    fixed-size-block.md
    linked-list.md

Related architecture documents:

    ../memory/memory.md
    ../memory/frame-allocator.md
    ../kernel_init.md

---

## Contributor Notes

- Heap pages must be mapped before initialization.
- Fixed-size and fallback regions must remain non-overlapping.
- Preserve allocator alignment guarantees.
- Keep allocator metadata private to allocator code.
- Changes to heap layout should be reflected in documentation.
- Keep allocator logic independent of process-management and scheduler subsystems.

---

## License

MIT or Apache 2.0 — TBD.