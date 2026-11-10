## Bulldog Kernel – Allocator Design

Reviewed: November 2026

This document describes the design philosophy, invariants, and architectural reasoning behind Bulldog's hybrid kernel heap allocator.

It ties together:

    allocator.rs
    fixed_size_block.rs
    linked_list.rs

and explains how the allocator subsystem maintains:

    Deterministic behaviour
    Low fragmentation
    Architectural clarity
    Extensibility
    Safety

The allocator subsystem is foundational to:

    Kernel initialization
    Memory management
    ELF loading
    VFS structures
    Process management
    User-mode support
    Future scheduler development

This document explains why Bulldog uses a hybrid allocator architecture and how the subsystem is intended to evolve.

---

## 1. Design Goals

Bulldog's allocator is designed around the following goals.

### Deterministic Behaviour

Kernel allocations must remain predictable during:

    Early boot
    Kernel initialization
    Process loading
    Filesystem initialization

Deterministic behaviour simplifies debugging and fault analysis.

---

### Low Fragmentation

Small allocations should not fragment the heap or cause allocation quality to degrade over time.

Most kernel allocations are relatively small and repetitive.

---

### Fast Common Allocations

Common kernel allocations should be:

    O(1)
    Predictable
    Cache-friendly

This is achieved through fixed-size allocation classes.

---

### General-Purpose Support

Large or unusual allocation sizes must still succeed without special-case logic.

The fallback allocator exists for this purpose.

---

### In-Place Metadata

Allocator metadata is stored inside allocator-managed memory.

Advantages:

    Minimal overhead
    Self-contained allocator state
    No external bookkeeping structures

---

### Clear Subsystem Boundaries

Allocator logic remains independent of:

    Syscalls
    VFS
    ELF loader
    Scheduler
    Process lifecycle

This keeps the subsystem reusable and testable.

---

## 2. Hybrid Allocator Architecture

Bulldog uses a two-tier heap architecture.

Current layout:

    [ Fixed-Size Region ]
    [ Fallback Region   ]

The allocator combines:

    FixedSizeBlockAllocator
    LockedHeap fallback allocator

Each tier is optimized for different allocation patterns.

---

## 3. Tier 1 – Fixed-Size Block Allocator

Located in:

    kernel/src/allocator/fixed_size_block.rs

Purpose:

    Fast path allocations
    Common allocation sizes
    Deterministic behaviour

Current block sizes:

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
    Minimal fragmentation
    Predictable performance

Suitable for:

    Kernel objects
    Metadata structures
    Small buffers
    Internal bookkeeping

---

## 4. Tier 2 – Fallback Allocator

Bulldog currently uses:

    linked_list_allocator::LockedHeap

as the fallback allocator.

Purpose:

    Large allocations
    Unusual layouts
    Requests larger than 4096 bytes

Characteristics:

    General-purpose allocation
    Variable-size allocations
    Dynamic region management

The fallback allocator is only used when fixed-size allocation is unsuitable.

---

## 5. LinkedListAllocator Reference Implementation

Bulldog also includes:

    kernel/src/allocator/linked_list.rs

This implementation is a standalone linked-list allocator.

Characteristics:

    In-place metadata
    Region splitting
    Region reinsertion
    Deterministic behaviour

Purpose:

    Allocator experimentation
    Reference implementation
    Specialized future uses

It is not currently used as the fallback allocator for the global heap.

---

## 6. Region Splitting Strategy

The heap is divided into two independent regions.

Current layout:

    First Half
        Fixed-size allocator

    Second Half
        Fallback allocator

Current heap size:

    2 MiB

Current strategy:

    1 MiB fixed-size region
    1 MiB fallback region

Advantages:

    No overlap
    Simple debugging
    Predictable behaviour
    Independent allocator state

Future versions may introduce dynamic sizing.

---

## 7. Alignment Strategy

### Fixed-Size Blocks

Fixed-size block classes use:

    alignment = block size

This guarantees:

    Correct alignment
    Free-list safety
    No padding requirements

---

### Fallback Allocator

Allocation layouts are adjusted to ensure:

    Adequate alignment
    Metadata storage
    Safe deallocation

This prevents regions from becoming unusable.

---

## 8. Heap Initialization

The allocator is initialized by:

    init_heap()

Current initialization sequence:

    Map heap pages
            ↓
    Align heap start
            ↓
    Split heap region
            ↓
    Initialize fallback allocator
            ↓
    Populate free-lists
            ↓
    Expose global allocator

This process occurs exactly once during startup.

---

## 9. Integration With Kernel Initialization

Allocator initialization occurs after:

    Paging initialization
    HHDM availability
    Frame allocator initialization

and before:

    FAT32 initialization
    VFS initialization
    User-mode execution
    Process creation

This ensures all higher-level subsystems have dynamic allocation capabilities.

---

## 10. Safety Invariants

The allocator subsystem relies on strict invariants.

### Heap Pages Must Be Mapped

The allocator assumes:

    Valid virtual mappings
    Writable heap memory

before initialization begins.

---

### Regions Must Not Overlap

Fixed-size and fallback regions must remain completely separate.

Violation causes allocator corruption.

---

### Metadata Must Remain Valid

Free-list nodes are stored inside allocator-managed memory.

Corruption of allocator metadata will break allocation behaviour.

---

### Initialization Occurs Exactly Once

Repeated initialization causes free-list corruption.

The heap must never be reinitialized.

---

### Alignment Must Be Preserved

Alignment rules are critical for:

    Free-list safety
    Metadata correctness
    Allocation validity

---

## 11. Fragmentation Strategy

### Fixed-Size Allocator

Fragmentation is avoided by using:

    Uniform block sizes
    Dedicated size classes

There is effectively no internal fragmentation within a free-list.

---

### Fallback Allocator

Fragmentation control depends on:

    Region splitting
    Region reuse
    Heap management by LockedHeap

Future improvements may introduce more advanced policies.

---

## 12. Relationship To User Mode

The kernel allocator currently serves:

    Kernel subsystems only

User-mode allocation is not yet implemented.

Future support may include:

    sys_alloc
    sys_free
    User heaps
    Per-process allocation regions

These will build on top of the existing memory-management infrastructure.

---

## 13. Future Extensions

### Short-Term

    Allocation statistics
    Debug allocation tracking
    Heap poisoning
    Guard pages

### Mid-Term

    Slab allocator integration
    Per-CPU heaps
    Kernel object caches

### Long-Term

    User-mode heap support
    VFS buffer caches
    NUMA-aware allocation
    Advanced allocation policies

---

## 14. Design Philosophy Summary

Bulldog's allocator subsystem is built on:

### Simplicity

Simple implementations are preferred over highly optimized but difficult-to-debug designs.

### Predictability

Common allocations should have deterministic behaviour.

### Safety

Strict invariants reduce allocator corruption risks.

### Performance

Fast-path allocation is optimized for common kernel workloads.

### Extensibility

Future allocators can be layered on top of the current implementation without redesigning the entire subsystem.

---

## Related Documentation

    README.md
    allocator.md
    fixed-size-block.md
    linked-list.md

Related architecture documents:

    ../memory/memory.md
    ../memory/frame-allocator.md
    ../kernel_init.md

---

## Contributor Notes

- Keep allocator behaviour deterministic.
- Maintain separation between fixed-size and fallback regions.
- Preserve allocator invariants.
- Review allocator documentation whenever heap layout changes.
- Keep allocator logic independent of process-management and scheduler code.

---

## License

MIT or Apache 2.0 — TBD.