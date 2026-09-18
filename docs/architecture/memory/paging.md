# Bulldog Kernel – Paging

This document describes Bulldog’s paging subsystem: how page tables are created, how virtual
addresses are mapped, and how the kernel manages memory through the x86_64 paging hardware.

It complements:

- `page-table.md` (structure details)
- `memory-map.md` (virtual layout)
- `frame-allocator.md` (physical frames)
- `hhdm.md` (direct physical map)

---

## 1. Paging Mode

Bulldog runs in:

- 64‑bit long mode  
- 4‑level paging (PML4 → PDPT → PD → PT)  
- 4 KiB pages  
- Optional 2 MiB huge pages (future)  

---

## 2. Page Table Hierarchy

```
PML4  → 512 entries
PDPT  → 512 entries
PD    → 512 entries
PT    → 512 entries
```

Each entry maps:

- A 4 KiB page  
- Or a 2 MiB huge page (future)  

---

## 3. Kernel Page Tables

Kernel page tables include:

- Higher‑half kernel mapping  
- Direct physical map (HHDM)  
- Kernel heap pages  
- APIC / MMIO mappings  
- Temporary identity map (removed after init)  

---

## 4. Mapping Pages

Mapping is performed via:

```rust
mapper.map_to(page, frame, flags, frame_allocator)
```

Flags include:

- `PRESENT`  
- `WRITABLE`  
- `NO_EXECUTE` (future)  
- `USER_ACCESSIBLE` (for user mode)  

---

## 5. Page Faults

Page faults occur when:

- Accessing unmapped memory  
- Writing to read‑only memory  
- Executing non‑executable memory  
- Accessing kernel memory from user mode  

Bulldog will implement a page fault handler in a future milestone.

---

## 6. Use Cases

### Kernel Heap
Maps heap pages during `init_heap`.

### ELF Loader
Maps user‑mode segments.

### VFS (future)
Maps file buffers.

### Process Model (future)
Creates per‑process page tables.

---

## 7. Future Enhancements

- Huge pages (2 MiB)  
- Copy‑on‑write  
- Guard pages  
- Page fault handler  
- Demand paging  
- Shared memory regions  
- Per‑CPU page tables  

---

## License
MIT or Apache 2.0 — to be determined.
