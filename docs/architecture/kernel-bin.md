## Bulldog Kernel – kernel-bin Architecture

Reviewed: November 2026

This document describes the purpose and architecture of the kernel-bin crate.

The Bulldog project separates the executable kernel image from the main kernel implementation.

Current kernel-related crates:

    boot-proto
    bootloader
    kernel
    kernel-bin

Each component has a specific responsibility.

---

## Overview

The kernel-bin crate is the executable entry point loaded by the Bulldog bootloader.

It is responsible for:

    Receiving BootInfo
    Performing early diagnostics
    Initializing paging
    Passing control to the kernel crate

The majority of operating-system functionality resides in the kernel crate.

The kernel-bin crate remains intentionally small.

---

## Architecture

Current startup path:

    UEFI Firmware
            ↓
    bootloader
            ↓
    BootInfo
            ↓
    kernel-bin
            ↓
    kernel_main()
            ↓
    kernel subsystem initialization

The kernel-bin crate exists to separate executable startup concerns from kernel implementation logic.

---

## Responsibilities

Current responsibilities include:

    Kernel entry point
    BootInfo validation
    Early serial diagnostics
    Paging initialization
    Transfer to kernel_main()

The crate does not implement:

    Filesystems
    Process management
    Syscalls
    Scheduling
    Memory allocation logic

These responsibilities belong to the kernel crate.

---

## kernel_entry()

Execution begins at:

    kernel_entry()

This symbol is exported as the executable entry point.

The linker script places this function into:

    .text.entry_point

ensuring the bootloader can locate and transfer control to the kernel.

---

## BootInfo Reception

The bootloader transfers control using:

    kernel_entry(&mut BootInfo)

The structure contains:

    Framebuffer information
    Memory map information
    HHDM information
    FAT image information
    Kernel metadata

kernel-bin acts as the first consumer of this information.

---

## Early Diagnostics

Before entering the kernel crate, kernel-bin performs a small amount of diagnostic logging.

Current diagnostics include:

    RSP value
    Framebuffer address
    Framebuffer virtual address
    Physical memory offset

These logs assist with:

    Boot debugging
    Paging verification
    HHDM validation
    Framebuffer troubleshooting

---

## Paging Initialization

Before the main kernel executes:

    paging::init()

is invoked.

This establishes the paging environment required by the kernel.

Current responsibilities include:

    Paging verification
    Framebuffer mapping validation
    HHDM integration

Paging initialization occurs before kernel_main().

---

## Transition To kernel_main()

After diagnostics and paging initialization:

    kernel_main(boot_info)

is called.

At this point responsibility transfers fully to the kernel crate.

The kernel crate then performs:

    Heap initialization
    FAT32 initialization
    VFS initialization
    Syscall initialization
    APIC initialization
    User-mode execution
    Process management

---

## Linker Script

The kernel-bin crate supplies its own linker script:

    kernel-bin/layout.ld

Current layout:

    Entry point
    Text section
    Read-only data
    Data
    BSS
    Stack section

The linker script ensures a predictable executable image layout for the bootloader.

---

## Entry Symbol

The executable entry point is:

    kernel_entry

The linker script specifies:

    ENTRY(kernel_entry)

making it the first code executed after bootloader handoff.

---

## Relationship To The kernel Crate

The kernel crate contains the operating-system implementation.

The kernel-bin crate contains executable startup glue.

Responsibilities:

kernel-bin:

    Entry point
    Diagnostics
    Early paging
    Handoff

kernel:

    Memory management
    Filesystems
    Syscalls
    APIC
    User mode
    Process management
    Future scheduling

This separation keeps startup code small and easier to debug.

---

## Current Status

Implemented:

    BootInfo handoff
    Entry diagnostics
    Paging initialization
    kernel_main() transfer

Future changes are expected to remain minimal.

Most architectural growth will occur in:

    kernel
    bootloader

rather than in kernel-bin itself.

---

## Contributor Notes

- Keep kernel-bin small.
- Avoid moving kernel subsystem logic into kernel-bin.
- Use kernel-bin only for early startup responsibilities.
- Any changes to BootInfo should be reviewed against boot-proto and the bootloader.
- Preserve serial diagnostics unless superseded by a better debugging mechanism.

---

## License

MIT or Apache 2.0 — TBD.