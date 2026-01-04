# Bulldog Kernel – Memory Management Overview

This document describes Bulldog’s memory management model, including paging, higher‑half
kernel layout, physical memory mapping, and early allocator considerations. It provides
contributors with a clear understanding of how memory is structured and accessed within the
`x86_64-bulldog` architecture.

Bulldog uses a higher‑half kernel design with identity‑mapped regions for early boot and
explicit physical memory management for kernel subsystems.

---

## Memory Model Overview

Bulldog’s memory model is built around the following principles:

- Higher‑half kernel mapping  
- Identity‑mapped low memory for early boot  
- Explicit physical memory management  
- Page‑table isolation between kernel and user space  
- Deterministic layout for debugging and contributor clarity  

The kernel runs in the canonical upper half of the virtual address space, while user space
occupies the lower half.

---

## Virtual Address Layout

A simplified view of Bulldog’s virtual address space:

```
+------------------------------+  0xFFFF_FFFF_FFFF_FFFF  (canonical high)
| Higher-Half Kernel Space     |
| - Kernel text/data           |
| - Kernel heap                |
| - Kernel stacks              |
| - Direct physical map        |
+------------------------------+  0xFFFF_8000_0000_0000
| Reserved / Guard Regions     |
+------------------------------+
| User Space (future)          |
| - User text/data             |
| - User heap/stack            |
| - Shared libraries (future)  |
+------------------------------+  0x0000_0000_0000_0000
```

This layout ensures:

- Kernel memory is isolated from user mode  
- Physical memory can be accessed via a direct map  
- Debugging is simplified due to stable address ranges  

---

## Paging Structure

Bulldog uses standard x86_64 four‑level paging:

- PML4  
- PDPT  
- PD  
- PT  

Key properties:

- Kernel mappings are global and shared across all processes  
- User mappings (future) will be per‑process  
- Large pages (2 MiB) may be used for kernel regions  
- Page tables are allocated from early physical memory  

---

## Higher‑Half Kernel Mapping

The kernel is linked to run at a higher‑half virtual address (typically above
`0xFFFF_8000_0000_0000`). During boot:

1. The bootloader identity‑maps low memory  
2. The kernel’s higher‑half mapping is installed  
3. Execution jumps to the higher‑half entry point  

This ensures:

- Kernel addresses are stable  
- User space cannot access kernel memory  
- Physical memory can be mapped into a predictable region  

---

## Direct Physical Memory Map

Bulldog maintains a direct mapping of physical memory into a fixed virtual region.  
This allows the kernel to convert between physical and virtual addresses without walking
page tables.

Example:

```
phys_addr + DIRECT_MAP_OFFSET = virt_addr
virt_addr - DIRECT_MAP_OFFSET = phys_addr
```

This region is used for:

- Frame allocation  
- Page‑table manipulation  
- DMA buffers (future)  
- Kernel heap initialization  

---

## Physical Memory Management

Bulldog uses a simple physical memory allocator during early boot:

- Memory regions are discovered via bootloader‑provided memory maps  
- Usable regions are inserted into a free‑frame list  
- Allocations return 4 KiB frames  
- Deallocations return frames to the free list  

Future enhancements:

- Bitmap‑based allocator  
- NUMA awareness  
- Large‑page support  
- Per‑CPU frame caches  

---

## Kernel Heap

The kernel heap is built on top of the physical allocator and direct map.  
It provides:

- Dynamic memory allocation for kernel subsystems  
- A safe Rust interface for internal use  
- Deterministic behavior for debugging  

Future improvements:

- Slab allocator  
- Guard pages  
- Leak detection  
- Allocation profiling  

---

## Guard Pages

Bulldog uses unmapped guard pages to detect:

- Stack overflows  
- Heap corruption  
- Invalid pointer dereferences  

Guard pages are placed:

- Below kernel stacks  
- Around critical kernel structures  
- At the edges of the kernel heap (future)  

---

## User‑Mode Memory (Future)

User‑mode memory will include:

- Per‑process page tables  
- User stacks and heaps  
- Copy‑on‑write regions  
- Memory‑mapped files  
- Shared memory segments  

Privilege switching and syscalls already lay the groundwork for this subsystem.

---

## Contributor Notes

- Keep memory layout diagrams updated as the kernel evolves  
- Document all new mappings and address ranges  
- Ensure page‑table changes are logged during early development  
- Validate alignment and guard pages for all kernel stacks  
- Avoid assumptions about physical memory ordering  

---

## License

MIT or Apache 2.0 — to be determined.
