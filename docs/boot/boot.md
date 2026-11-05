## Bulldog Kernel – Boot and Higher-Half Initialization

Reviewed: November 2026

This document describes Bulldog's current boot process as implemented by the UEFI 0.41 bootloader and current kernel architecture.

It reflects the behavior of:

    bootloader/src/main.rs
    bootloader/src/boot_mod.rs
    bootloader/src/paging.rs
    boot-proto/src/lib.rs

    kernel/src/lib.rs
    kernel/src/memory.rs
    kernel/src/gdt.rs
    kernel/src/interrupts.rs
    kernel/src/apic.rs
    kernel/src/process.rs

This document reflects the current implementation rather than future design goals.

---

## 1. Boot Overview

Bulldog's current boot sequence is:

    UEFI Firmware
            ↓
    Bootloader startup
            ↓
    GOP initialization
            ↓
    Framebuffer console creation
            ↓
    kernel.elf loaded
            ↓
    fat.img loaded
            ↓
    BootInfo populated
            ↓
    ExitBootServices()
            ↓
    Identity map + HHDM created
            ↓
    CR3 loaded
            ↓
    kernel_main()
            ↓
    kernel_init()
            ↓
    Heap initialization
            ↓
    FAT32 initialization
            ↓
    VFS initialization
            ↓
    Syscall initialization
            ↓
    User program loading
            ↓
    User-mode execution
            ↓
    Process management
            ↓
    Kernel idle loop

This sequence is deterministic and extensively logged through the serial logging subsystem.

---

## 2. Bootloader Responsibilities

The bootloader is responsible for:

    Loading kernel.elf
    Loading fat.img
    Initializing GOP
    Constructing BootInfo
    Gathering the memory map
    Building identity mappings
    Building the Higher-Half Direct Map (HHDM)
    Installing CR3
    Providing a kernel stack
    Jumping to the kernel

The kernel assumes these responsibilities have already been completed when execution begins.

---

## 3. BootInfo Handoff

The kernel receives a BootInfo structure containing:

### Framebuffer Information

    Physical framebuffer address
    Virtual framebuffer address
    Width
    Height
    Stride
    Pixel format

### Memory Information

    Physical memory offset (HHDM)
    Memory region table
    Memory region count

### Kernel Information

    Kernel physical start address
    Kernel physical end address
    Kernel entry address

### Filesystem Information

    FAT image physical address
    FAT image size

BootInfo is the primary contract between bootloader and kernel.

---

## 4. Entering kernel_main()

Execution enters:

    kernel_main(&mut BootInfo)

The kernel performs:

    Framebuffer initialization
    Writer initialization
    Logger initialization
    Memory map extraction
    FAT image registration

Control is then transferred to:

    kernel_init()

where the majority of subsystem initialization occurs.

---

## 5. Higher-Half Addressing

Bulldog operates using a Higher-Half Direct Map.

Current base:

    0xFFFF800000000000

Address conversion:

    virtual = physical + HHDM_BASE

This allows direct access to physical memory through higher-half virtual addresses.

Used for:

    Page-table management
    Frame allocator access
    Framebuffer access
    Hardware mappings
    Memory management

---

## 6. Memory Initialization

Using the BootInfo memory map, the kernel constructs:

    Pre-heap allocator
    Heap allocator
    Frame allocator

These allocators become the foundation for:

    Dynamic allocation
    VFS structures
    File descriptor tables
    User-mode memory mappings

The allocator subsystem must be operational before higher-level services can be initialized.

---

## 7. Heap Initialization

The kernel initializes the heap using:

    allocator::init_heap()

After heap initialization:

    Vec becomes usable
    Box becomes usable
    FAT structures become usable
    VFS becomes usable
    Process tables become usable

Heap initialization is one of the earliest kernel responsibilities.

---

## 8. FAT32 Initialization

The bootloader loads fat.img into memory before kernel startup.

The kernel:

    Reads the BPB
    Loads the FAT
    Registers global FAT structures
    Reads the root directory

Current user applications are loaded from this FAT32 image.

Example:

    /bin/hello
    /bin/goodbye

---

## 9. Virtual File System (VFS)

After FAT32 initialization the VFS is initialized.

Current usage:

    ELF loading
    File operations
    User-mode program access

The VFS provides a filesystem abstraction independent of the underlying storage implementation.

---

## 10. GDT, TSS and IST Stacks

The kernel initializes:

    GDT
    TSS
    Interrupt Stack Tables (IST)

Current IST usage includes:

    Double fault handling
    Page fault handling
    LAPIC timer interrupt handling

Requirements:

    Must occur before interrupts are enabled
    Stacks must remain mapped and aligned

---

## 11. Interrupt Descriptor Table

The kernel installs:

    CPU exception handlers
    Page-fault handler
    Double-fault handler
    LAPIC timer handler
    Spurious interrupt handler

The IDT is loaded before interrupts are enabled.

---

## 12. Syscall Initialization

The syscall subsystem installs:

    Vector 0x80

Current interface uses:

    rax
    rdi
    rsi
    rdx

User-mode processes already make use of this mechanism.

Current implemented syscall categories include:

    Process exit
    Memory allocation
    File operations
    File descriptor handling

---

## 13. Local APIC Initialization

Bulldog configures the Local APIC during boot.

Initialization includes:

    LAPIC discovery
    Version reporting
    ID reporting
    SVR configuration
    LAPIC timer setup

Timer configuration:

    Periodic mode
    Divisor = 16
    Initial count = 500000

Diagnostic information is logged through the serial logger.

---

## 14. User-Mode Initialization

Bulldog currently supports user-mode execution.

The kernel can:

    Load ELF executables
    Map user segments
    Allocate user stacks
    Enter Ring 3
    Service syscalls
    Return to kernel mode

Verified programs:

    /bin/hello
    /bin/goodbye

---

## 15. Process Management Integration

After ELF loading:

    Process records are created
    PIDs are assigned
    Entry addresses are tracked
    User stack addresses are tracked

Current process states:

    Ready
    Running
    Exited

Verified lifecycle:

    Ready
      ↓
    Running
      ↓
    Exited

Process records persist after exit for diagnostics and future scheduler integration.

---

## 16. Interrupt Enable

Once initialization is complete:

    interrupts::enable()

is executed.

At this point:

    Heap is active
    Frame allocator is active
    FAT32 is active
    VFS is active
    APIC is active
    Syscalls are active
    User-mode support is available

---

## 17. Kernel Idle State

When no further work remains, Bulldog enters:

    hlt_loop()

or returns there following completion of user-mode execution.

This state represents the current end of the boot process.

---

## 18. Future Enhancements

Planned work includes:

    Scheduler
    Context switching
    SMP support
    IOAPIC initialization
    ACPI integration
    KASLR

User-mode execution and process management are already implemented and are no longer future work.

---

## Contributor Notes

- BootInfo is the foundation of kernel initialization.
- HHDM assumptions must remain synchronized with the bootloader.
- FAT image handoff is a required part of the current boot process.
- User-mode support is operational and should be documented as implemented.
- Process management is operational and forms the basis of future scheduler work.

---

## License

MIT or Apache 2.0 — TBD.