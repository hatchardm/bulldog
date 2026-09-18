# Bulldog Kernel – Memory Map (Physical & Virtual)

This document describes Bulldog’s physical and virtual memory layout, including the higher‑half
kernel mapping, direct physical map (HHDM), user‑mode layout, and memory region classification.

It complements:

- `memory.md` (paging implementation)
- `layout.md` (kernel ELF layout)
- `boot/bootloader.md` (memory map handoff)

---

## 1. Physical Memory Layout

The physical memory map is provided by the bootloader via UEFI.

Typical regions:

| Region Type        | Description |
|--------------------|-------------|
| Usable RAM         | Allocatable physical frames |
| Reserved           | Firmware, bootloader, kernel image |
| ACPI Reclaimable   | ACPI tables |
| ACPI NVS           | Sleep state memory |
| MMIO               | Device memory |
| Loader Data        | Bootloader allocations |

The kernel converts these into `MemoryRegion` entries.

---

## 2. Kernel Physical Load Address

The kernel ELF is loaded at a physical address such as:

```
0x0010_0000
```

This region is identity‑mapped during early boot and later mapped into the higher half.

---

## 3. Virtual Memory Layout

Bulldog uses a higher‑half kernel design:

```
0x0000_0000_0000_0000 ────────────────────────────────
    User space
    Code, data, heap, stack
──────────────────────────────────────────────────────
0x0000_FFFF_FFFF_FFFF ────────────────────────────────
    Reserved (canonical boundary)
──────────────────────────────────────────────────────
0xFFFF_8000_0000_0000 ────────────────────────────────
    Higher‑half kernel
    Kernel text, data, heap
──────────────────────────────────────────────────────
0xFFFF_9000_0000_0000 ────────────────────────────────
    Direct physical map (HHDM)
──────────────────────────────────────────────────────
0xFFFF_FFFF_FFFF_FFFF
```

### Higher‑Half Kernel Region

Kernel ELF sections are mapped here:

- `.text` → RX  
- `.rodata` → R  
- `.data` → RW  
- `.bss` → RW (zeroed)

### Direct Physical Map (HHDM)

Maps all physical memory contiguously:

```
virtual = physical + HHDM_OFFSET
```

This enables:

- Fast frame allocator  
- Simple MMIO access  
- Deterministic physical addressing

---

## 4. User‑Mode Layout

User processes receive:

- Code/data segments from ELF loader  
- Heap (future)  
- Stack at top of user space  
- Per‑process page table  
- No access to kernel space

---

## 5. Kernel Page Tables

Kernel page tables include:

- Higher‑half kernel mapping  
- Direct physical map  
- Global mappings (APIC, MMIO)  
- Temporary identity map (removed after init)

---

## 6. Future Enhancements

- Guard pages for kernel stacks  
- Per‑CPU memory regions  
- KASLR  
- NUMA awareness  
- Memory‑mapped files (VFS integration)

---

## Contributor Notes

- Keep memory layout diagrams updated  
- Document all new mappings  
- Maintain canonical address boundaries  
- Avoid mixing user and kernel mappings

---

## License
MIT or Apache 2.0 — to be determined.
