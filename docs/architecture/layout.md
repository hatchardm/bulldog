# Bulldog Kernel – Memory Layout Specification

This document defines Bulldog’s physical and virtual memory layout, including the higher‑half
kernel region, direct physical map, identity‑mapped boot regions, and reserved address
ranges. It serves as a reference for contributors working on paging, allocators, privilege
switching, and future user‑mode execution.

Bulldog targets the `x86_64-bulldog` architecture and uses a higher‑half kernel design with
a deterministic virtual address structure.

---

## Overview

Bulldog’s memory layout is designed around the following goals:

- Stable higher‑half kernel addresses  
- Deterministic virtual address regions  
- Direct physical memory mapping  
- Identity‑mapped low memory for early boot  
- Clear separation between kernel and user space  
- Contributor‑friendly debugging and logging  

This layout is consistent across all kernel builds and is documented here for clarity.

---

## Virtual Address Space Layout

A simplified view of Bulldog’s virtual address space:

```
0xFFFF_FFFF_FFFF_FFFF  ────────────────────────────────  Canonical high
|                     Higher-Half Kernel Space          |
|                                                       |
|  Kernel text / rodata                                 |
|  Kernel data / BSS                                    |
|  Kernel heap                                          |
|  Kernel stacks                                        |
|  Direct physical map                                  |
|                                                       |
0xFFFF_8000_0000_0000  ────────────────────────────────  Kernel base
|                     Reserved / Guard Regions          |
|                                                       |
0xFFFF_0000_0000_0000  ────────────────────────────────  Reserved (future)
|                     Unused / Reserved                 |
|                                                       |
0x0000_8000_0000_0000  ────────────────────────────────  User/kernel boundary
|                     User Space (future)               |
|                                                       |
|  User text / data                                     |
|  User heap / stack                                    |
|  Shared libraries (future)                            |
|                                                       |
0x0000_0000_0000_0000  ────────────────────────────────  Canonical low
```

Key properties:

- Kernel occupies the upper canonical half  
- User space occupies the lower canonical half  
- No overlap between user and kernel regions  
- Direct physical map resides inside the higher‑half region  

---

## Kernel Virtual Regions

### Kernel Text and Read‑Only Data

Located at the beginning of the higher‑half region:

```
0xFFFF_8000_0000_0000 → 0xFFFF_8000_XXXX_XXXX
```

Contains:

- `.text`  
- `.rodata`  
- `.gcc_except_table` (if present)  

Mapped as:

- Read‑only  
- Executable  

---

### Kernel Data and BSS

Immediately following `.text`:

```
0xFFFF_8000_XXXX_XXXX → 0xFFFF_8000_YYYY_YYYY
```

Contains:

- `.data`  
- `.bss`  
- Global kernel structures  

Mapped as:

- Read/write  
- Non‑executable  

---

### Kernel Heap

The kernel heap grows upward from a fixed base:

```
0xFFFF_8080_0000_0000 → dynamic growth
```

Backed by:

- Physical frames from the early allocator  
- Later: slab allocator, guard pages, profiling  

---

### Kernel Stacks

Each CPU (future SMP) receives:

- A dedicated kernel stack  
- A guard page below the stack  

Example layout:

```
[ Guard Page ]
[ Kernel Stack ]
[ Per-CPU Data (future) ]
```

Stacks reside in a reserved higher‑half region.

---

## Direct Physical Memory Map

Bulldog maps all physical memory into a contiguous virtual region:

```
virt = phys + DIRECT_MAP_OFFSET
phys = virt - DIRECT_MAP_OFFSET
```

Example:

```
DIRECT_MAP_OFFSET = 0xFFFF_9000_0000_0000
```

This region is used for:

- Frame allocation  
- Page‑table manipulation  
- DMA buffers (future)  
- Kernel heap initialization  

Mapped as:

- Read/write  
- Non‑executable  

---

## Identity‑Mapped Boot Region

During early boot, the bootloader provides identity mapping for low memory:

```
0x0000_0000_0000_0000 → 0x0000_0000_0000_0000
```

Used for:

- Bootloader stack  
- Early kernel entry  
- Temporary page tables  
- Boot information structures  

This region is discarded once the kernel transitions fully into the higher half.

---

## Physical Memory Layout

A typical physical memory map:

```
0x0000_0000 – 0x0009_FFFF   → Low memory / BIOS remnants (unused)
0x0010_0000 – 0x00FF_FFFF   → Kernel ELF load region
0x0100_0000 – ...           → Usable RAM (frame allocator)
ACPI / MMIO regions         → Reserved
```

Bulldog uses the bootloader‑provided memory map to determine:

- Usable RAM  
- Reserved regions  
- ACPI tables  
- MMIO windows  

---

## Reserved Regions

Bulldog reserves the following virtual regions:

- `0xFFFF_0000_0000_0000` → `0xFFFF_7FFF_FFFF_FFFF`  
  Reserved for future kernel subsystems  

- `0x0000_8000_0000_0000` → `0xFFFF_0000_0000_0000`  
  User/kernel boundary  

- Guard pages around:  
  - kernel stacks  
  - kernel heap (future)  
  - critical kernel structures  

---

## Future Layout Extensions

Planned additions:

- Per‑process virtual address spaces  
- Copy‑on‑write regions  
- Shared memory segments  
- Memory‑mapped files  
- User‑mode stacks and heaps  
- KASLR (kernel address randomization)  
- Per‑CPU direct‑map windows  

---

## Contributor Notes

- Keep this layout updated as new subsystems are added  
- Document all new virtual regions and address ranges  
- Ensure mappings remain deterministic for debugging  
- Validate alignment and guard pages for all kernel stacks  
- Avoid overlapping or ambiguous regions  

---

## License

MIT or Apache 2.0 — to be determined.
