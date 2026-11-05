## Bulldog Kernel – Boot & Initialization Subsystem

Reviewed: November 2026

This directory contains documentation for Bulldog's bootloader and early kernel initialization subsystems.

These documents describe the current UEFI 0.41 boot architecture and the initialization path used by the Bulldog kernel before normal execution begins.

The boot subsystem forms the foundation of the operating system and is responsible for establishing the execution environment required by all higher-level components.

Current documented areas include:

- UEFI boot services
- Graphics Output Protocol (GOP)
- BootInfo construction
- Memory map handoff
- Higher-Half Direct Mapping (HHDM)
- Paging bootstrap
- Kernel handoff
- Early memory initialization
- FAT32 initialization
- VFS initialization
- Syscall initialization
- User-mode activation
- Process management integration

---

## Documents

### Bootloader Architecture

    bootloader.md

Describes the current UEFI 0.41 bootloader implementation.

Topics include:

    GOP initialization
    Framebuffer handoff
    FAT image loading
    BootInfo construction
    Memory-map processing
    HHDM creation
    Paging activation
    Kernel transfer

---

### Kernel Boot Process

    boot.md

Describes the kernel-side initialization sequence after bootloader handoff.

Topics include:

    kernel_main()
    kernel_init()
    Heap initialization
    Frame allocator construction
    FAT32 initialization
    VFS initialization
    GDT/TSS setup
    IDT setup
    Syscall setup
    LAPIC initialization
    User-mode startup
    Process management integration

---

## Purpose of This Layer

The boot layer defines the earliest execution environment used by Bulldog.

Responsibilities include:

    Firmware interaction
    Memory discovery
    Framebuffer initialization
    Paging setup
    BootInfo handoff
    Kernel startup
    Early subsystem initialization

This layer exists to provide a stable foundation for:

    Architecture layer
    Interface layer
    Filesystem layer
    User-mode execution
    Process management
    Future scheduler development

---

## Current Architecture

The current startup sequence is:

    UEFI Firmware
            ↓
    Bootloader
            ↓
    BootInfo Handoff
            ↓
    kernel_main()
            ↓
    kernel_init()
            ↓
    Memory Initialization
            ↓
    FAT32 Initialization
            ↓
    VFS Initialization
            ↓
    Syscall Initialization
            ↓
    User Mode
            ↓
    Process Management
            ↓
    Kernel Idle

---

## Related Layers

### Architecture Layer

Hardware-facing subsystems:

    ../architecture/index.md

Includes:

    Interrupts
    APIC
    Memory Management
    GDT
    TSS
    CPU Initialization

---

### Interface Layer

User-mode and syscall interfaces:

    ../interface/index.md

Includes:

    Syscalls
    Process Interfaces
    ELF Loading
    Scheduler Planning
    VFS Interfaces

---

## Contributor Notes

- Keep boot documentation synchronized with bootloader and kernel code.
- Treat BootInfo as the authoritative bootloader-to-kernel contract.
- Document all changes to memory layout and paging assumptions.
- Update these documents whenever bootloader responsibilities change.
- Keep implementation details accurate following UEFI version migrations.

---

## Current Status

Implemented:

    UEFI 0.41 bootloader
    GOP framebuffer support
    BootInfo handoff
    FAT image handoff
    HHDM creation
    Heap initialization
    FAT32 support
    VFS support
    ELF loading
    User-mode execution
    Process management

Planned:

    ACPI integration
    IOAPIC support
    SMP startup
    Scheduler implementation
    Context switching
    KASLR

---

## License

MIT or Apache 2.0 — TBD.