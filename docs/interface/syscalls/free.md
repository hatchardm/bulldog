# Bulldog Kernel – `free` Syscall

Frees memory previously allocated by `alloc`.

---

## 1. Prototype

```
free(ptr: *mut u8)
```

---

## 2. Behavior

- Frees memory  
- Pointer must be valid  
- Double‑free is undefined behavior  

---

## License
MIT or Apache 2.0 — TBD.
