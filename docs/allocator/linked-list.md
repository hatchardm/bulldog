# Bulldog Kernel – Linked‑List Allocator

This document describes the standalone linked‑list allocator implemented in:

```
kernel/src/allocator/linked_list.rs
```

This allocator manages free memory regions using a singly‑linked list. It is a general‑purpose
allocator suitable for contiguous regions of memory and is used internally by Bulldog for
specialized allocation tasks.

It is **not** the fallback allocator used by the global heap (that role is handled by
`linked_list_allocator::LockedHeap`). Instead, this allocator is a pure kernel implementation
with deterministic behavior and in‑place metadata.

---

## 1. Overview

The linked‑list allocator manages memory as a list of free regions. Each region is represented
by a `ListNode` stored directly inside the free memory itself.

Key properties:

- Metadata is stored *in place*  
- Free regions are kept in a singly‑linked list  
- Allocation searches for the first region that fits  
- Regions are split when partially used  
- Deallocation reinserts the region into the list  
- Alignment is guaranteed  
- No external bookkeeping structures are required  

This allocator is simple, deterministic, and suitable for early kernel subsystems.

---

## 2. Data Structures

### `ListNode`

```rust
struct ListNode {
    size: usize,
    next: Option<&'static mut ListNode>,
}
```

Each node describes:

- The size of the free region  
- A pointer to the next free region  

The node is stored at the beginning of the free region.

### Allocator Structure

```rust
pub struct LinkedListAllocator {
    head: ListNode,
}
```

`head.next` points to the first free region.

---

## 3. Initialization

The allocator must be initialized once:

```rust
unsafe fn init(&mut self, heap_start: usize, heap_size: usize)
```

This:

- Writes a `ListNode` into the region  
- Sets `head.next` to point to it  
- Marks the entire region as free  

Safety requirements:

- Region must be valid and writable  
- Region must not overlap other allocations  
- Region must be unused  
- Region must be large enough to hold a `ListNode`

---

## 4. Allocation

Allocation is performed via:

```rust
unsafe fn alloc(&self, layout: Layout) -> *mut u8
```

### Step 1 — Adjust layout

The allocator ensures:

- Alignment ≥ `align_of::<ListNode>()`  
- Size ≥ `size_of::<ListNode>()`  

This guarantees that freed regions can store a `ListNode`.

### Step 2 — Search free list

`find_region(size, align)` walks the list:

- Aligns the region start  
- Checks if region is large enough  
- Ensures remainder is large enough to hold a `ListNode`  
- Removes region from free list if suitable  

### Step 3 — Split region

If the region is larger than needed:

- The excess is reinserted as a new free region  
- Using `add_free_region(alloc_end, excess_size)`

### Step 4 — Return pointer

If no region fits, return `null_mut()`.

---

## 5. Deallocation

Deallocation reinserts the region:

```rust
unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout)
```

Steps:

1. Adjust layout (same as allocation)  
2. Write a `ListNode` into the freed region  
3. Insert region at the front of the free list  

This is a simple LIFO free‑list insertion.

---

## 6. Alignment Rules

Alignment is handled by:

```rust
align_up(addr, align)
```

Requirements:

- `align` must be a power of two  
- Region start must be aligned  
- Remainder must be large enough to hold a `ListNode`

If alignment causes the usable portion to shrink below `size_of::<ListNode>()`, the region is rejected.

---

## 7. Region Splitting

When a region is partially used:

```
[ region_start ---------------- region_end ]
[ alloc_start ---- alloc_end ][ excess ]
```

The excess region is reinserted via:

```rust
add_free_region(alloc_end, excess_size)
```

This ensures:

- No fragmentation leaks  
- Free list remains valid  
- All free regions contain valid metadata  

---

## 8. Safety Notes

The allocator relies on strict invariants:

- All free regions must contain valid `ListNode` metadata  
- Regions must not overlap  
- Caller must ensure pointer validity  
- Layout must match original allocation  
- `init` must be called exactly once  
- Region must be mapped and writable  

Violating these invariants can corrupt the free list.

---

## 9. Use Cases

This allocator is suitable for:

- Early kernel subsystems  
- Temporary memory pools  
- Specialized allocators (slab, object caches)  
- VFS buffers  
- ELF loader scratch space  
- Boot‑time memory management  

It is not used as the global heap — that role belongs to the hybrid allocator in
`allocator.rs`.

---

## 10. Future Extensions

- Sorted free list (best‑fit)  
- Coalescing adjacent free regions  
- Debug mode with region poisoning  
- Statistics and fragmentation metrics  
- Integration with kernel memory map visualizer  

---

## License
MIT or Apache 2.0 — to be determined.
