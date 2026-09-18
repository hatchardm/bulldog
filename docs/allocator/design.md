# Bulldog Kernel – Allocator Design

This document describes the design philosophy, invariants, and architectural reasoning behind
Bulldog’s hybrid kernel heap allocator. It ties together the fixed‑size block allocator, the
linked‑list allocator, the fallback heap, and the initialization logic in `allocator.rs`.

The allocator subsystem is foundational to:

- Kernel initialization  
- Paging and memory management  
- Syscall `alloc` / `free` (future)  
- ELF loader  
- VFS buffers  
- Scheduler and process creation  
- Kernel object allocation  

This document explains *why* Bulldog uses a hybrid design and *how* the subsystem maintains
safety, determinism, and architectural clarity.

---

## 1. Goals

Bulldog’s allocator is designed around the following goals:

### **Deterministic behavior**
Kernel allocations must be predictable, especially during early boot and subsystem
initialization.

### **Low fragmentation**
Small allocations should not fragment the heap or degrade performance over time.

### **Fast path for common sizes**
Most kernel allocations are small (8–512 bytes). These should be O(1).

### **General‑purpose fallback**
Large or unusual allocations must still succeed without special‑case logic.

### **In‑place metadata**
Allocator metadata should live inside the heap itself, not in global structures.

### **Clear subsystem boundaries**
Allocator logic must remain independent of:

- Syscalls  
- VFS  
- ELF loader  
- Scheduler  
- Process model  

This keeps the allocator reusable and testable.

---

## 2. Hybrid Allocator Architecture

Bulldog uses a **two‑tier hybrid heap**:

```
[ fixed-size block region ] [ fallback region ]
```

### Tier 1 — Fixed‑Size Block Allocator (Fast Path)

Located in:

```
kernel/src/allocator/fixed_size_block.rs
```

Provides O(1) allocation for block sizes:

```
8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096 bytes
```

Characteristics:

- Deterministic  
- No fragmentation  
- Alignment guaranteed  
- Perfect for kernel objects, small buffers, and control structures  
- Free‑lists stored directly in the heap  

### Tier 2 — Linked‑List Fallback Allocator (Slow Path)

Two variants exist:

#### 1. `linked_list_allocator::LockedHeap`  
Used as the fallback region in the global allocator.

#### 2. Bulldog’s own `LinkedListAllocator`  
Located in:

```
kernel/src/allocator/linked_list.rs
```

Used for specialized allocation tasks.

Characteristics:

- Handles large allocations  
- Splits free regions  
- Reinserts freed regions  
- Stores metadata in‑place  
- Deterministic and simple  

---

## 3. Region Splitting Strategy

The heap is split into two regions:

```
fixed-size region = first half
fallback region   = second half
```

This ensures:

- Fixed‑size blocks never overlap fallback allocations  
- Fallback allocator has a contiguous region  
- Free‑lists remain valid  
- Metadata never corrupts other subsystems  

The split is simple but effective. Future versions may use:

- Dynamic region sizing  
- Growth heuristics  
- Per‑CPU heaps  
- NUMA‑aware allocation  

---

## 4. Alignment and In‑Place Metadata

### Fixed‑Size Blocks

Alignment is trivial:

```
block_size == alignment
```

This guarantees:

- Every block can store a `ListNode`  
- No misalignment bugs  
- No padding required  

### Linked‑List Allocator

The allocator adjusts layouts:

```
size >= size_of::<ListNode>()
align >= align_of::<ListNode>()
```

This ensures:

- Freed regions can store metadata  
- Remainders can be split safely  
- No region becomes “too small to free”  

---

## 5. Safety Invariants

The allocator subsystem relies on strict invariants:

### **1. Heap pages must be mapped before initialization**
`init_heap()` must run after paging is active.

### **2. Fixed‑size and fallback regions must not overlap**
Caller guarantees non‑overlap.

### **3. All free regions must contain valid metadata**
Both allocators write `ListNode` structures into free memory.

### **4. Layout must match original allocation**
Deallocation must use the same layout.

### **5. Initialization must occur exactly once**
Reinitializing the allocator corrupts the free list.

### **6. Alignment must be preserved**
Misaligned blocks break the free‑list structure.

### **7. No subsystem may write into allocator metadata**
Allocator regions must be treated as opaque.

---

## 6. Integration with Kernel Initialization

The allocator is initialized in:

```
init_heap(mapper, frame_allocator)
```

Steps:

1. Map heap pages  
2. Align heap start  
3. Split region  
4. Initialize fallback allocator  
5. Carve fixed‑size blocks  
6. Expose global allocator  

This occurs after:

- Paging  
- Frame allocator  
- Higher‑half transition  

But before:

- Syscall initialization  
- VFS  
- ELF loader  
- Scheduler  
- Process creation  

This ensures all subsystems have a working heap.

---

## 7. Interaction with Future Syscalls

Future syscalls:

- `sys_alloc`  
- `sys_free`  
- `sys_realloc` (optional)

Will wrap the global allocator.

User‑mode heaps will be:

- Per‑process  
- Backed by per‑process page tables  
- Allocated via kernel frame allocator  
- Managed by a user‑mode allocator (dlmalloc, mimalloc, or custom)

Kernel allocator remains separate.

---

## 8. Fragmentation Strategy

### Fixed‑Size Blocks
No fragmentation — blocks are uniform.

### Linked‑List Allocator
Fragmentation is controlled by:

- In‑place metadata  
- Region splitting  
- Remainder reinsertion  
- Alignment rules  

Future enhancements may include:

- Coalescing adjacent free regions  
- Best‑fit search  
- Region merging  
- Slab allocator for kernel objects  

---

## 9. Future Extensions

### Short‑Term
- Allocation statistics  
- Debug mode with poisoning  
- Guard pages around heap  

### Mid‑Term
- Slab allocator for kernel objects  
- Per‑CPU heaps  
- NUMA‑aware allocation  

### Long‑Term
- User‑mode heap  
- Memory‑mapped file buffers  
- VFS cache allocator  
- Kernel object pools  

---

## 10. Design Philosophy Summary

Bulldog’s allocator subsystem is built on:

### **Simplicity**
No unnecessary complexity. Deterministic behavior.

### **Predictability**
Fixed‑size blocks handle the majority of allocations.

### **Safety**
Strict invariants prevent corruption.

### **Performance**
Fast path for small allocations; fallback for large ones.

### **Extensibility**
Clear boundaries allow future allocators to be layered on top.

---

## License
MIT or Apache 2.0 — to be determined.
