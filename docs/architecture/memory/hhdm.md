# Bulldog Kernel – Higher‑Half Direct Map (HHDM)

The Higher‑Half Direct Map (HHDM) provides a contiguous virtual mapping of all physical
memory. It allows the kernel to access physical memory directly without performing page table
walks or maintaining separate mapping structures.

This document explains Bulldog’s HHDM design, invariants, and integration.

---

## 1. Purpose

HHDM provides:

- Direct access to physical memory  
- Fast frame allocator operations  
- Simple MMIO access  
- Deterministic mapping for drivers  
- A stable base for paging and kernel heap  

It is a core part of Bulldog’s higher‑half architecture.

---

## 2. Mapping Formula

Bulldog uses:

```
virtual = physical + HHDM_OFFSET
```

Where:

```
HHDM_OFFSET = 0xFFFF_9000_0000_0000
```

This offset places the direct map above the higher‑half kernel region.

---

## 3. Layout

```
0xFFFF_8000_0000_0000 ────────────────────────────────
    Higher‑half kernel (ELF image)
──────────────────────────────────────────────────────
0xFFFF_9000_0000_0000 ────────────────────────────────
    Direct physical map (HHDM)
──────────────────────────────────────────────────────
0xFFFF_FFFF_FFFF_FFFF
```

The direct map covers:

- All usable RAM  
- ACPI regions  
- MMIO regions (optional)  
- Bootloader structures (optional)  

---

## 4. Initialization

HHDM is created during early paging setup:

1. Kernel builds new page tables  
2. Maps physical memory into HHDM region  
3. Switches CR3  
4. Jumps into higher‑half kernel  

After this point:

- Identity map is removed  
- Kernel runs entirely in higher‑half  
- All physical memory is accessible via HHDM  

---

## 5. Safety Invariants

- HHDM must map all physical memory contiguously  
- HHDM must not overlap kernel image  
- HHDM must not overlap user space  
- HHDM must not overlap MMIO mappings unless intentional  
- HHDM must be present before heap initialization  
- HHDM must remain stable for the lifetime of the kernel  

---

## 6. Use Cases

### Frame Allocator
Converts physical frame addresses into virtual pointers.

### Kernel Heap
Uses HHDM to access heap pages.

### ELF Loader
Copies ELF segments using HHDM pointers.

### VFS (future)
Maps file buffers and block device memory.

### Drivers
Access MMIO regions via HHDM.

---

## 7. Future Enhancements

- KASLR (randomize HHDM offset)  
- Per‑CPU HHDM regions  
- NUMA‑aware HHDM segmentation  
- MMIO mapping via HHDM  

---

## License
MIT or Apache 2.0 — to be determined.
