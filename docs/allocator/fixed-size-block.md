# Bulldog Kernel – Fixed‑Size Block Allocator

This document describes the fixed‑size block allocator implemented in:

```
kernel/src/allocator/fixed_size_block.rs
```

It provides fast, deterministic allocation for common block sizes and forms the first tier of
Bulldog’s hybrid heap.

---

## 1. Block Sizes

Supported block sizes:

```
8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096 bytes
```

Each size class has a free‑list:

```rust
list_heads: [Option<&'static mut ListNode>; BLOCK_SIZES.len()]
```

---

## 2. Allocation Flow

### Step 1 — Determine size class
```
list_index(layout)
```

Chooses the smallest block ≥ required size or alignment.

### Step 2 — Fast path
If a free block exists:

- Pop from free‑list  
- Return block pointer  

### Step 3 — Slow path
If free‑list is empty:

- Allocate a block from fallback allocator  
- Using a layout equal to the block size  

---

## 3. Deallocation Flow

### Step 1 — Determine size class  
Same logic as allocation.

### Step 2 — Fast path  
If block fits a size class:

- Write a `ListNode` into the block  
- Push block onto free‑list  

### Step 3 — Slow path  
If block does not fit any size class:

- Forward to fallback allocator  

---

## 4. Region Carving

`populate_fixed_lists` walks the fixed‑size region:

- Aligns each block  
- Creates a `ListNode`  
- Pushes it onto the appropriate free‑list  
- Continues until region exhausted

This produces a deterministic block layout.

---

## 5. Safety Notes

- Caller guarantees region is mapped and non‑overlapping  
- `ListNode` fits inside every block size  
- Alignment is preserved  
- Free‑lists must not overlap fallback region  

---

## 6. Future Extensions

- Per‑CPU free‑lists  
- Slab allocator for kernel objects  
- Debug mode with block poisoning  
- Allocation statistics  

---

## License
MIT or Apache 2.0 — to be determined.
