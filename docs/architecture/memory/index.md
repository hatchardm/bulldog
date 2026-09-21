# Bulldog Kernel – Memory Subsystem Index

The memory subsystem defines how Bulldog manages physical memory, virtual memory, paging
structures, higher‑half mapping, and the direct physical map (HHDM). It is one of the core
hardware‑facing components of the architecture layer.

This directory contains documentation for all memory‑related architecture components.

---

## Contents

### **1. Memory Overview**
`memory.md`  
High‑level description of Bulldog’s memory architecture, including:

- Higher‑half kernel design  
- Virtual address layout  
- Physical memory model  
- Paging fundamentals  
- Kernel heap integration  
- Guard pages (future)  

---

### **2. Memory Map**
`memory-map.md`  
Detailed description of:

- Physical memory map from the bootloader  
- Region classification (usable, reserved, ACPI, MMIO)  
- Virtual memory layout  
- Higher‑half kernel region  
- Direct physical map (HHDM)  
- Canonical address boundaries  

This document complements the bootloader handoff documentation in:

```
../../boot/bootloader.md
```

---

### **3. Frame Allocator (Planned)**
`frame-allocator.md` (future)  
Will describe:

- Physical frame allocation  
- Integration with HHDM  
- Free‑list or bitmap allocator design  
- Interaction with paging and kernel heap  
- Safety invariants  

This document will be added once the frame allocator is finalized.

---

### **4. HHDM (Higher‑Half Direct Map)**
`hhdm.md` (future)  
Will describe:

- Virtual → physical mapping formula  
- Why Bulldog uses a direct map  
- How the kernel accesses physical memory  
- MMIO mapping strategy  
- Integration with APIC, ACPI, and paging  

This document will be added when HHDM documentation is expanded.

---

## Purpose of the Memory Subsystem

The memory subsystem provides:

- Deterministic physical memory management  
- Safe virtual memory abstraction  
- Kernel‑only higher‑half execution  
- Direct physical access for allocators and drivers  
- Foundation for paging, heap, VFS, ELF loader, and processes  

It is tightly integrated with:

- `boot/bootloader.md`  
- `architecture/layout.md`  
- `architecture/kernel_init.md`  
- `allocator/` subsystem  
- Future process and VFS subsystems  

---

## Contributor Notes

- Keep memory documentation technically precise  
- Update this index when adding new memory documents  
- Maintain canonical address boundaries  
- Document all new mappings and invariants  
- Avoid mixing user‑mode memory with kernel memory documentation  

---

## License
MIT or Apache 2.0 — to be determined.
