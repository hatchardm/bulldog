# Bulldog Kernel – `alloc` Syscall

Allocates memory in the process heap.

---

## 1. Prototype

```
alloc(size: usize) -> *mut u8
```

Returns pointer to allocated memory or null on failure.

---

## 2. Behavior

- Allocates `size` bytes  
- Uses kernel heap allocator  
- Returns pointer in user space  
- Memory is zero‑initialized  

---

## 3. Errors

- Out of memory → returns null  

---

## License
MIT or Apache 2.0 — TBD.
