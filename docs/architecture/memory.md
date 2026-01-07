Got you, Mark — here is the entire rewritten memory.md in a single raw code block, with no formatting broken, ready for direct paste into your repo.
# Bulldog Kernel – Memory Management (Current Implementation)

This document describes Bulldog’s *actual* memory management implementation as of the current kernel state. It reflects the real behavior of:

- `memory.rs` (frame allocators, bitmap, paging helpers)
- `gdt.rs` (TSS, IST stacks)
- `apic.rs` (LAPIC MMIO mapping)
- `kernel_init` (boot‑time initialization sequence)

This is not a future design document — it is a precise description of how Bulldog works today.

---

# 1. Overview

Bulldog currently implements:

- A higher‑half kernel memory layout  
- A direct physical memory mapping (via bootloader offset)  
- Early physical frame allocation  
- A bitmap‑based frame tracking system  
- LAPIC MMIO mapping into higher‑half space  
- GDT + TSS with dedicated IST stacks  
- Basic paging helpers (`map_page`, `map_lapic_mmio`)  

User‑mode memory, guard pages, and advanced allocators are **not yet implemented**.

---

# 2. Virtual Address Layout (Current)

Bulldog uses a higher‑half kernel model. The bootloader provides a physical‑to‑virtual offset that is used to access page tables and physical memory.

```
+------------------------------+  0xFFFF_FFFF_FFFF_FFFF
| Higher-Half Kernel Space     |
| - Kernel text/data           |
| - Kernel heap                |
| - Kernel stacks              |
| - LAPIC MMIO region          |  (0xFFFF_FF00_0000_0000)
| - Direct physical map        |
+------------------------------+  0xFFFF_8000_0000_0000
| Reserved / Guard Regions     |
+------------------------------+
| (User space not implemented) |
+------------------------------+  0x0000_0000_0000_0000
```

### Key regions implemented today

| Region | Address | Notes |
|--------|---------|-------|
| LAPIC MMIO | `0xFFFF_FF00_0000_0000` | Mapped to physical `0xFEE0_0000` |
| Direct map | bootloader‑provided offset | Used for page‑table access |
| Kernel stacks | allocated statically | Used by TSS IST entries |

---

# 3. Paging Infrastructure

Bulldog uses the standard x86_64 4‑level paging model.

### Active Page Table Access

`active_level_4_table()` uses the physical memory offset to convert the CR3 physical address into a virtual address inside the direct map.

### Offset Page Table

`init_offset_page_table()` constructs an `OffsetPageTable` using the bootloader’s offset.

### Mapping Helpers

- `map_page()` — maps a single 4 KiB page  
- `map_lapic_mmio()` — maps the LAPIC MMIO region into higher‑half space  

Large pages are **not** used yet.

---

# 4. LAPIC MMIO Mapping

The Local APIC is mapped into higher‑half kernel space at:

```
LAPIC_VIRT_BASE = 0xFFFF_FF00_0000_0000
```

`map_lapic_mmio()` maps:

- Virtual: `LAPIC_VIRT_BASE`
- Physical: `0xFEE0_0000`
- Flags: `PRESENT | WRITABLE | NO_EXECUTE`

`apic.rs` then uses this mapping for:

- LAPIC ID
- LAPIC version
- Timer configuration
- End‑of‑interrupt (EOI)

This mapping is required before calling `setup_apic()`.

---

# 5. Physical Memory Management

Bulldog currently uses **two allocators** depending on boot stage.

---

## 5.1 Pre‑Heap Allocator

`PreHeapAllocator` provides up to **512 frames** during early boot.

- Backed by a fixed array: `[Option<PhysFrame>; 512]`
- Used before the heap is initialized
- Returned via `init_temp()` in `BootInfoFrameAllocator`

This allocator is simple and deterministic.

---

## 5.2 BootInfoFrameAllocator

After the heap is available, Bulldog switches to `BootInfoFrameAllocator`.

### Features implemented today

- Stores all discovered frames in a `Vec<PhysFrame>`
- Tracks allocated frames using a bitmap (`FrameBitmap`)
- Provides `allocate_frame()` for 4 KiB frames
- Provides `mark_used_frames()` to mark non‑usable regions

### Bitmap Details

`FrameBitmap`:

- 32,768 bytes → tracks 262,144 frames (1 GiB)
- Hard‑coded base address: `0x100000`
- Tracks **used** frames only
- No deallocation yet
- No dynamic resizing

This is a partial implementation of a future full bitmap allocator.

---

# 6. Interrupt Stacks (IST) and TSS

`gdt.rs` defines a TSS with **two dedicated IST stacks**:

| IST Index | Purpose | Backing Stack |
|-----------|---------|---------------|
| 0 | Double fault | `STACK` |
| 1 | LAPIC timer + page fault | `LAPIC_STACK` |

Each stack:

- Is 128 KiB  
- Is 16‑byte aligned  
- Lives in higher‑half kernel space  
- Is statically allocated  

Guard pages are **not implemented yet**, but planned.

The GDT contains:

- Kernel code segment  
- Kernel data segment  
- TSS descriptor  

`init()` loads:

- GDT  
- CS/DS/ES/SS  
- TSS  

---

# 7. Kernel Initialization Flow (Current)

The actual initialization sequence is:

1. Bootloader loads kernel and provides memory map + physical offset  
2. Kernel constructs `OffsetPageTable`  
3. Pre‑heap allocator is created via `init_temp()`  
4. LAPIC MMIO region is mapped  
5. GDT + TSS are initialized  
6. LAPIC is configured (`setup_apic()`)  
7. Full `BootInfoFrameAllocator` is created  
8. Kernel heap is initialized (external to this document)  

This sequence reflects the real code paths in `lib.rs`, `memory.rs`, `gdt.rs`, and `apic.rs`.

---

# 8. Missing or Future Features

These features are mentioned in earlier design notes but **not yet implemented**:

- Guard pages around stacks and heap  
- User‑mode memory  
- Per‑process page tables  
- Copy‑on‑write  
- Memory‑mapped files  
- Large‑page support  
- NUMA awareness  
- Frame deallocation  
- Per‑CPU frame caches  

These will be added after syscalls and privilege switching are complete.

---

# 9. Contributor Notes

- The memory subsystem is evolving; this document reflects the current state.  
- LAPIC mapping must occur before enabling the LAPIC timer.  
- IST stacks must remain aligned and unmoved.  
- The bitmap allocator currently tracks **used** frames only.  
- Hard‑coded limits (1 GiB bitmap) will be removed in future revisions.  
- Avoid assumptions about future user‑mode memory until the syscall subsystem is complete.

---

# License

MIT or Apache 2.0 — TBD.
