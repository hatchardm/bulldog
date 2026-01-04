# Bulldog Kernel – Linker Script and ELF Layout

This document describes Bulldog’s kernel ELF layout, linker script structure, section
placement, and higher‑half virtual address mapping. It provides contributors with a clear
understanding of how the kernel binary is organized in memory and how the linker ensures
correct placement of code, data, and metadata.

Bulldog targets the `x86_64-bulldog` architecture and uses a higher‑half linking strategy
that maps the kernel into the upper canonical half of the virtual address space.

---

## Overview

Bulldog’s linker script defines:

- The kernel’s higher‑half virtual base address  
- The physical load address used by the bootloader  
- Placement of ELF sections (`.text`, `.rodata`, `.data`, `.bss`)  
- Alignment and padding rules  
- Symbols exported for use by the kernel (e.g., section boundaries)  
- Memory regions used during boot and runtime  

The linker script is a critical part of the kernel’s boot and memory architecture.

---

## Higher‑Half Linking Strategy

Bulldog is linked to run at a virtual address above:

```
0xFFFF_8000_0000_0000
```

This ensures:

- Kernel memory is isolated from user space  
- Kernel addresses are stable and deterministic  
- Paging can map the kernel into a fixed region  
- Direct physical mapping can coexist with kernel code  

The kernel is **loaded physically** at a low address (e.g., `0x0010_0000`) but **executes
virtually** in the higher half.

---

## Physical Load Address

The bootloader loads the kernel ELF into a physical region such as:

```
0x0010_0000  (1 MiB)
```

This region contains:

- ELF headers  
- Program segments  
- Kernel code and data  

The linker script ensures that:

- Virtual addresses are in the higher half  
- Physical addresses remain low  
- The bootloader can map physical → virtual correctly  

---

## Virtual Address Layout (Linker View)

The linker defines the following virtual layout:

```
KERNEL_BASE = 0xFFFF_8000_0000_0000

.text   → KERNEL_BASE + 0x0000_0000
.rodata → KERNEL_BASE + 0x00XX_XXXX
.data   → KERNEL_BASE + 0x00YY_YYYY
.bss    → KERNEL_BASE + 0x00ZZ_ZZZZ
```

All sections are page‑aligned to simplify paging and debugging.

---

## ELF Sections

### `.text`

Contains:

- Kernel code  
- Interrupt handlers  
- Syscall entry stubs  
- Early boot routines  

Mapped as:

- Read‑only  
- Executable  

---

### `.rodata`

Contains:

- Constant data  
- Jump tables  
- Read‑only metadata  

Mapped as:

- Read‑only  
- Non‑executable  

---

### `.data`

Contains:

- Initialized global variables  
- Kernel structures requiring write access  

Mapped as:

- Read/write  
- Non‑executable  

---

### `.bss`

Contains:

- Zero‑initialized globals  
- Static buffers  

The kernel entry point zeroes this region before enabling interrupts.

---

## Linker‑Defined Symbols

The linker script exports symbols used by the kernel:

```
_start
_etext
_sdata
_edata
_sbss
_ebss
_kernel_start
_kernel_end
```

These symbols are used for:

- BSS zeroing  
- Heap initialization  
- Memory map validation  
- Debugging and logging  

---

## Alignment Rules

Bulldog enforces:

- 4 KiB alignment for all sections  
- 2 MiB alignment for large kernel regions (future)  
- 16‑byte alignment for stack frames  
- Page alignment for direct‑map regions  

This ensures compatibility with paging and simplifies debugging.

---

## Program Headers

The kernel ELF contains:

- One loadable segment for `.text` and `.rodata`  
- One loadable segment for `.data` and `.bss`  

This minimizes bootloader complexity and ensures predictable memory layout.

---

## Interaction with Paging

The linker script defines **virtual** addresses.  
Paging maps these virtual addresses to physical memory.

Example mapping:

```
virt: 0xFFFF_8000_0000_0000  → phys: 0x0010_0000
virt: 0xFFFF_8000_0020_0000  → phys: 0x0030_0000
```

The bootloader or early kernel code installs these mappings before jumping to the higher‑half entry point.

See:  
`boot.md`  
`memory.md`  
`layout.md`

---

## Debugging Notes

- Kernel addresses in logs always reflect higher‑half virtual addresses  
- Physical addresses are used only during early boot and allocator setup  
- GDB can be configured with the kernel’s virtual base for symbolic debugging  
- Section boundaries (`_etext`, `_edata`, etc.) are essential for diagnosing memory issues  

---

## Future Enhancements

Planned improvements:

- KASLR (kernel address randomization)  
- Separate `.init` and `.fini` sections  
- Per‑CPU linker‑defined regions  
- Dedicated section for syscall stubs  
- Fine‑grained memory permissions  

---

## Contributor Notes

- Keep the linker script aligned with the memory layout specification  
- Document all new sections and exported symbols  
- Ensure alignment rules remain consistent with paging  
- Avoid introducing unnecessary segments or relocations  
- Validate that all sections map cleanly into the higher‑half region  

---

## License

MIT or Apache 2.0 — to be determined.
