## Bulldog Kernel – Linker and Executable Image Layout

Reviewed: November 2026

This document describes the current linker architecture used by Bulldog's executable kernel image.

It reflects the implementation found in:

    kernel-bin/layout.ld
    kernel-bin/src/main.rs

The linker script controls the layout of the final executable image loaded by the Bulldog bootloader.

---

## 1. Overview

Bulldog separates:

    bootloader
    boot-proto
    kernel
    kernel-bin

The final executable loaded by the bootloader is produced by:

    kernel-bin

The linker script defines:

    Entry point
    Section ordering
    Load address
    Stack placement

The script intentionally remains simple.

---

## 2. Startup Path

The current startup sequence is:

    UEFI Firmware
            ↓
    bootloader
            ↓
    kernel-bin image
            ↓
    kernel_entry()
            ↓
    paging::init()
            ↓
    kernel_main()

The linker script ensures the CPU begins execution at:

    kernel_entry

after bootloader handoff.

---

## 3. Entry Point

The linker script declares:

    ENTRY(kernel_entry)

This symbol becomes the first code executed inside the kernel image.

Current implementation:

    kernel-bin/src/main.rs

contains:

    kernel_entry()

which receives:

    &mut BootInfo

from the bootloader.

---

## 4. Load Address

The image currently begins at:

    0x00100000

(1 MiB)

The linker script establishes:

    . = 1M;

All following sections are placed relative to this location.

The bootloader loads the executable image into memory and transfers control to the entry point.

---

## 5. Section Layout

Current section order:

    .text.entry_point
    .text
    .rodata
    .data
    .bss
    .stack

This layout keeps startup code at the beginning of the image and places runtime data afterward.

---

## 6. Entry Section

The linker script preserves:

    .text.entry_point

using:

    KEEP(*(.text.entry_point))

The kernel entry function is explicitly placed into this section.

Purpose:

    Stable startup location
    Guaranteed inclusion
    Clear handoff target

Without KEEP(), the linker could potentially discard the section during optimization.

---

## 7. Text Section

The .text section contains executable code.

Current contents include:

    kernel_entry()
    kernel crate code
    interrupt handlers
    syscall handlers
    supporting runtime code

The linker places:

    *(.text*)

into this section.

---

## 8. Read-Only Data

The .rodata section contains:

    String literals
    Constant tables
    Static read-only values

Current rule:

    *(.rodata*)

---

## 9. Initialized Data

The .data section contains:

    Initialized globals
    Writable runtime data

Current rule:

    *(.data*)

---

## 10. Uninitialized Data

The .bss section contains:

    Zero-initialized globals
    Static buffers
    Runtime storage

Current rule:

    *(.bss*)
    *(COMMON)

---

## 11. Stack Section

The linker script defines:

    .stack

aligned to:

    4 KiB

Current rule:

    .stack ALIGN(4K)

This provides a predictable location for statically allocated kernel stack structures.

---

## 12. Alignment

Current alignment guarantees:

    4 KiB stack alignment

Future alignment requirements may expand as memory-management features evolve.

The current script deliberately avoids unnecessary complexity.

---

## 13. Relationship To kernel-bin

The linker script exists specifically for:

    kernel-bin

Responsibilities:

    Define image layout
    Define entry point
    Group executable sections

It does not contain operating-system logic.

---

## 14. Relationship To The kernel Crate

The kernel crate provides:

    Memory management
    Filesystems
    Syscalls
    APIC support
    Usermode support
    Process management

The linker script merely determines how this code is arranged in the executable image.

---

## 15. Interaction With The Bootloader

The bootloader:

    Loads the image
    Builds BootInfo
    Initializes paging
    Transfers control

The linker script provides the structure required for that handoff.

Current transfer path:

    bootloader
            ↓
    kernel_entry()
            ↓
    kernel_main()

---

## 16. Current Design Philosophy

The current linker script intentionally prioritizes:

    Simplicity
    Predictability
    Debuggability

Bulldog currently uses a straightforward executable layout rather than a highly optimized or heavily segmented kernel image.

---

## 17. Future Enhancements

Possible future additions:

    Dedicated .init section
    Dedicated .fini section
    Per-CPU regions
    Link-time validation
    KASLR support
    Additional protection-oriented sections

These are not currently implemented.

---

## Contributor Notes

- Keep linker changes simple and well documented.
- Preserve the kernel_entry() entry point.
- Preserve .text.entry_point placement.
- Update documentation whenever new sections are added.
- Review linker changes alongside bootloader changes.
- Validate startup behaviour after linker modifications.

---

## License

MIT or Apache 2.0 — TBD.