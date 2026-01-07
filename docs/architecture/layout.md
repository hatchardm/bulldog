# Bulldog Kernel – Memory Layout (Current Implementation)

> **Disclaimer:**  
> This document describes Bulldog’s *current* memory layout as implemented in the kernel today.  
> It does **not** represent the full long‑term design.  
> Planned features such as guard pages, user‑mode address spaces, IOAPIC regions, per‑CPU stacks,  
> and KASLR will be documented once implemented.  
>  
> For subsystem‑specific details, see:  
> - `memory.md` (paging, allocators, direct map)  
> - `gdt.md` (TSS, IST stacks)  
> - `apic.md` (LAPIC MMIO region)  
> - `kernel_init.md` (initialization sequence)

This file provides a high‑level view of the virtual and physical memory layout used by Bulldog today.  
It reflects the real behavior of the kernel and bootloader, not the future architecture.

---

# 1. Overview

Bulldog uses a higher‑half kernel design with:

- A stable higher‑half kernel region  
- A direct physical memory map (bootloader‑provided offset)  
- Statically allocated kernel stacks (including IST stacks)  
- A fixed LAPIC MMIO virtual address  
- No user‑mode memory yet  
- No guard pages yet  

This layout is deterministic and optimized for debugging and contributor clarity.

---

# 2. Virtual Address Space (Current)

```
0xFFFF_FFFF_FFFF_FFFF  ────────────────────────────────  Canonical high
|                     Higher-Half Kernel Space          |
|                                                       |
|  Kernel text / rodata                                 |
|  Kernel data / BSS                                    |
|  Kernel heap                                          |
|  Kernel stacks                                        |
|  LAPIC MMIO region (0xFFFF_FF00_0000_0000)            |
|  Direct physical map (bootloader offset)              |
|                                                       |
0xFFFF_8000_0000_0000  ────────────────────────────────  Kernel base
|                     Reserved (unused)                 |
|                                                       |
0x0000_0000_0000_0000  ────────────────────────────────  Canonical low
```

### Key points:

- Kernel occupies the upper canonical half  
- User space is **not implemented**  
- Direct map uses the offset provided by the bootloader  
- LAPIC MMIO is mapped at a fixed higher‑half address  
- Kernel stacks (including IST stacks) live in higher‑half static regions  

---

# 3. Kernel Virtual Regions

### Kernel Text / Read‑Only Data

Loaded by the bootloader into higher‑half memory.

Mapped as:

- Read‑only  
- Executable  

### Kernel Data / BSS

Mapped as:

- Read/write  
- Non‑executable  

### Kernel Heap

Initialized during `kernel_init` using:

- Temporary pre‑heap allocator  
- Full bitmap allocator  

Grows upward from a fixed region in higher‑half space.

### Kernel Stacks

Bulldog currently uses:

- A main kernel stack  
- Two IST stacks:  
  - `STACK` (double fault)  
  - `LAPIC_STACK` (LAPIC timer + page fault)

All stacks:

- Are statically allocated  
- Are 128 KiB  
- Are 16‑byte aligned  
- Have **no guard pages yet**  

---

# 4. Direct Physical Memory Map

The bootloader provides a physical‑to‑virtual offset:

```
virt = phys + phys_mem_offset
```

This region is used for:

- Page‑table access  
- Frame allocation  
- LAPIC stack remapping  
- Logging physical memory regions  

Mapped as:

- Read/write  
- Non‑executable  

---

# 5. LAPIC MMIO Region

Mapped at a fixed virtual address:

```
0xFFFF_FF00_0000_0000
```

Mapped to physical:

```
0xFEE0_0000
```

Flags:

- PRESENT  
- WRITABLE  
- NO_EXECUTE  

Used by:

- `lapic_read` / `lapic_write`  
- `setup_apic()`  

---

# 6. Physical Memory Layout (Current)

A typical bootloader memory map includes:

```
0x0000_0000 – 0x0009_FFFF   → Low memory (unused by kernel)
0x0010_0000 – 0x00FF_FFFF   → Kernel ELF load region
0x0100_0000 – ...           → Usable RAM (frame allocator)
ACPI / MMIO regions         → Reserved
```

Bulldog uses:

- Bootloader memory map  
- `BootInfoFrameAllocator`  
- Bitmap tracking of used frames  

---

# 7. Reserved Regions (Current)

The following virtual regions are reserved but not yet used:

- `0xFFFF_0000_0000_0000` → `0xFFFF_7FFF_FFFF_FFFF`  
  Reserved for future kernel subsystems  

- `0x0000_0000_0000_0000` → `0xFFFF_7FFF_FFFF_FFFF`  
  User space (future)  

Guard pages are planned but not implemented.

---

# 8. Future Layout Extensions

Planned but **not implemented**:

- User‑mode address spaces  
- Per‑process page tables  
- Guard pages around stacks and heap  
- Copy‑on‑write  
- Shared memory  
- Memory‑mapped files  
- IOAPIC MMIO region  
- SMP per‑CPU stacks  
- KASLR  

These will be documented once implemented.

---

# 9. Contributor Notes

- This file describes the *current* layout, not the final design  
- See `memory.md`, `gdt.md`, `apic.md`, and `kernel_init.md` for subsystem details  
- Keep this document updated as new regions are added  
- Avoid introducing overlapping or ambiguous address ranges  
- Maintain deterministic mappings for debugging  

---

# License

MIT or Apache 2.0 — TBD.
