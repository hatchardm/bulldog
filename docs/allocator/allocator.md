# Bulldog Kernel – Heap Allocator (allocator.rs)

This document describes the top‑level kernel heap allocator defined in:

```
kernel/src/allocator.rs
```

It wires together the fixed‑size block allocator and the fallback linked‑list allocator,
maps the heap region, and exposes the global allocator used by all kernel allocations.

---

## 1. Heap Layout

The kernel heap is located at:

```
HEAP_START = 0x4444_4444_0000
HEAP_SIZE  = 100 KiB
```

This region is mapped during kernel initialization using the kernel’s page tables.

---

## 2. Initialization (`init_heap`)

`init_heap` performs:

### 1. Page Mapping
Each page in the heap range is mapped:

- Writable  
- Present  
- Backed by frames from the frame allocator  

### 2. Alignment
Heap start is aligned to:

```
MAX_BLOCK_ALIGN = 1024 bytes
```

This ensures fixed‑size blocks are properly aligned.

### 3. Region Split
The heap is divided into:

- **Fixed‑size region** (first half)  
- **Fallback region** (second half)  

### 4. Allocator Initialization
```
alloc.init_with_regions(fixed_start, fixed_size, fallback_start, fallback_size)
```

This:

- Initializes `LockedHeap` with the fallback region  
- Carves the fixed‑size region into blocks  

---

## 3. Global Allocator

The kernel exposes:

```rust
#[global_allocator]
static ALLOCATOR: Locked<FixedSizeBlockAllocator>
```

This means:

- All kernel allocations use the hybrid allocator  
- No subsystem needs to manually reference the allocator  
- Allocator is thread‑safe via `spin::Mutex`

---

## 4. Safety Notes

- Caller guarantees non‑overlapping regions  
- Heap pages must be mapped before initialization  
- Alignment must be preserved  
- Fallback allocator must not overlap fixed‑size blocks  

---

## 5. Future Extensions

- Heap resizing  
- Per‑CPU heaps  
- Slab allocator integration  
- User‑mode heap via syscalls  
- Guard pages for debugging  

---

## License
MIT or Apache 2.0 — to be determined.
