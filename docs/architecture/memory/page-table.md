# Bulldog Kernel – Page Table Structures

This document describes the structure and semantics of x86_64 page tables as used by Bulldog.
It complements `paging.md` by focusing on the data structures themselves.

---

## 1. Page Table Levels

### PML4 (Level 4)
- Top‑level table  
- Selects PDPT  
- Kernel uses high entries (≥256)  
- User space uses low entries (<256)

### PDPT (Level 3)
- Selects PD  
- Can map 1 GiB pages (not used)

### PD (Level 2)
- Selects PT  
- Can map 2 MiB pages (future)

### PT (Level 1)
- Maps 4 KiB pages  
- Most common mapping level  

---

## 2. Entry Format

Each entry contains:

- Physical address (bits 12–51)  
- Flags (bits 0–11)  
- Reserved bits (52–62)  
- NX bit (63)  

---

## 3. Flags

Common flags:

| Flag | Meaning |
|------|---------|
| PRESENT | Page is valid |
| WRITABLE | Writes allowed |
| USER_ACCESSIBLE | User mode can access |
| WRITE_THROUGH | Cache write‑through |
| CACHE_DISABLE | Disable caching |
| ACCESSED | Hardware‑set |
| DIRTY | Hardware‑set |
| HUGE_PAGE | 2 MiB or 1 GiB page |
| GLOBAL | TLB global page |
| NO_EXECUTE | Execution disabled |

Bulldog currently uses:

- `PRESENT`  
- `WRITABLE`  
- `USER_ACCESSIBLE` (user mode)  

---

## 4. Kernel vs User Space

### Kernel Space
- Higher‑half  
- Not user‑accessible  
- Contains kernel image, heap, HHDM, MMIO  

### User Space
- Lower half  
- Per‑process page tables  
- Contains code, data, heap, stack  

---

## 5. Page Table Creation

Page tables are created using:

```rust
OffsetPageTable
```

which provides:

- Safe Rust wrappers  
- Automatic TLB flush  
- Error handling (`MapToError`)  

---

## 6. Safety Invariants

- Kernel mappings must not be user‑accessible  
- HHDM must be contiguous  
- Page tables must be aligned to 4 KiB  
- Page tables must be zeroed before use  
- No overlapping mappings  
- No aliasing unless intentional  

---

## 7. Future Enhancements

- NX bit enforcement  
- Huge page support  
- Page table dumping tools  
- Page table visualizer  
- KASLR randomization  

---

## License
MIT or Apache 2.0 — to be determined.
