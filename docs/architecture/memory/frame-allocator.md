# Bulldog Kernel – Frame Allocator

This document describes Bulldog’s physical frame allocator: the component responsible for
handing out physical memory frames to the kernel, paging subsystem, heap allocator, and
future user‑mode processes.

It complements:

- `memory.md` (overview)
- `memory-map.md` (physical/virtual map)
- `hhdm.md` (direct physical map)
- `paging.md` (page table operations)

---

## 1. Purpose

The frame allocator provides:

- Physical frames for page table creation  
- Frames for kernel heap pages  
- Frames for mapping ELF segments  
- Frames for VFS buffers (future)  
- Frames for user‑mode address spaces (future)  

It is the lowest‑level memory allocator in the kernel.

---

## 2. Input: Bootloader Memory Map

The allocator is built from the UEFI memory map passed via `BootInfo`.

Usable regions:

- `MemoryRegionKind::Usable`  
- Size ≥ 2 MiB (Bulldog’s current constraint)  

The allocator skips:

- Firmware regions  
- ACPI regions  
- MMIO regions  
- Kernel image region  
- Bootloader allocations  

---

## 3. Allocation Strategy

Bulldog currently uses a **linear bump allocator** for frames:

```
next → next free physical address
end  → end of usable region
```

Allocation:

```
frame = next
next += 4096
```

If `next + 4096 > end`, allocation fails.

This strategy is:

- Simple  
- Deterministic  
- Safe during early boot  
- Sufficient for initial paging and heap setup  

Future versions will replace this with a bitmap allocator.

---

## 4. Safety Invariants

- Allocator must only hand out frames from usable RAM  
- Allocator must never overlap kernel image  
- Allocator must never overlap bootloader structures  
- Allocator must never hand out MMIO frames  
- Allocator must never hand out ACPI frames  
- Allocator must be initialized before paging  
- Allocator must not be reinitialized  

---

## 5. Integration Points

### Paging
Used by:

- `map_to`  
- Page table creation  
- Kernel heap mapping  
- ELF loader segment mapping  

### Kernel Heap
`init_heap()` allocates frames for heap pages.

### ELF Loader
Allocates frames for:

- `.text`  
- `.data`  
- `.bss`  
- Stack pages  

### Future Subsystems
- VFS buffer cache  
- Process address spaces  
- Shared memory  
- Slab allocator  

---

## 6. Future Enhancements

### Bitmap Allocator
Track free/used frames with a bitmap.

### Region Coalescing
Support multiple usable regions.

### NUMA Awareness
Per‑node frame pools.

### Guard Frames
Protect kernel stacks and sensitive regions.

### Frame Recycling
Support freeing frames (not currently possible).

---

## License
MIT or Apache 2.0 — to be determined.
