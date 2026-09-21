# Bulldog Kernel – Allocator Subsystem Overview

Bulldog’s allocator subsystem provides the kernel heap used by all dynamic memory
allocations (`Box`, `Vec`, `String`, collections, and internal kernel subsystems). It is a
hybrid design combining:

- A **fixed‑size block allocator** for fast, predictable small allocations  
- A **linked‑list fallback allocator** for general‑purpose heap usage  

This subsystem is initialized early in `kernel_init` and is completely syscall‑agnostic.  
User‑mode allocation will be layered on top of this design in future milestones.

---

## Components

### Fixed‑Size Block Allocator
Located in:

```
kernel/src/allocator/fixed_size_block.rs
```

Provides O(1) allocation for common block sizes:

```
8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096 bytes
```

Each size class has a free‑list. Blocks are carved from a dedicated region of the heap.

### Linked‑List Fallback Allocator
Located in:

```
kernel/src/allocator/linked_list.rs
```

Backed by `linked_list_allocator::LockedHeap`.  
Used when:

- No fixed‑size block fits  
- A free‑list is empty  
- Allocation requires >4096 bytes  

### Allocator Wiring
Located in:

```
kernel/src/allocator.rs
```

Responsible for:

- Mapping heap pages  
- Splitting heap into fixed‑size and fallback regions  
- Initializing both allocators  
- Exposing the global allocator via:

```
#[global_allocator]
static ALLOCATOR: Locked<FixedSizeBlockAllocator>
```

---

## Initialization Flow

1. Kernel maps heap pages (`init_heap`)  
2. Heap start is aligned  
3. Region is split into:
   - Fixed‑size block region  
   - Fallback region  
4. Fixed‑size region is carved into blocks  
5. Fallback region is given to `LockedHeap`  
6. Allocator becomes active globally

---

## Future Extensions

- User‑mode heap (`sys_alloc`, `sys_free`)  
- Per‑process heap regions  
- Guard pages  
- Slab allocator for kernel objects  
- NUMA‑aware allocation  
- VFS buffer cache integration  

---

## License
MIT or Apache 2.0 — to be determined.
