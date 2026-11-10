## Bulldog Kernel – Startup and Kernel Initialization

Reviewed: November 2026

This document describes the current startup path of the Bulldog kernel from entry into the kernel crate through subsystem initialization and optional user-mode execution.

It reflects the implementation found in:

    kernel/src/lib.rs

and related subsystems:

    memory.rs
    gdt.rs
    interrupts.rs
    apic.rs
    syscall.rs
    vfs.rs
    process.rs
    fs/*

This document describes the current implementation rather than future architecture plans.

---

## 1. Startup Overview

Bulldog startup currently follows this sequence:

    bootloader
            ↓
    kernel-bin
            ↓
    kernel_main()
            ↓
    kernel_init()
            ↓
    Heap Initialization
            ↓
    FAT32 Initialization
            ↓
    CPU Initialization
            ↓
    Syscalls
            ↓
    APIC
            ↓
    Interrupt Enable
            ↓
    VFS
            ↓
    File Descriptors
            ↓
    Optional User Mode
            ↓
    Idle Loop

The startup process is divided into two major stages:

    kernel_main()
    kernel_init()

Each stage has distinct responsibilities.

---

## 2. Responsibilities of kernel_main()

kernel_main() is responsible for preparing the kernel runtime environment.

Its responsibilities include:

    FAT image registration
    Framebuffer initialization
    Writer initialization
    Logger initialization
    Memory-map preparation
    Transfer to kernel_init()

The function acts as the bridge between bootloader handoff and subsystem initialization.

---

## 3. kernel_main() Initialization Sequence

Current flow:

    Receive BootInfo
            ↓
    Store FAT image information
            ↓
    Create framebuffer
            ↓
    Initialize writer
            ↓
    Initialize logger
            ↓
    Process memory information
            ↓
    Execute kernel_init()
            ↓
    Enter hlt_loop()

When kernel_init() returns the kernel enters:

    hlt_loop()

which represents the normal idle state.

---

## 4. FAT Image Registration

The bootloader passes:

    fat_phys_addr
    fat_size

through BootInfo.

kernel_main() stores these values globally for later filesystem initialization.

These values are later used to create:

    RamDisk

during FAT32 initialization.

---

## 5. Framebuffer Initialization

kernel_main() creates:

    KernelFramebuffer

using framebuffer information derived from BootInfo.

Current responsibilities:

    Runtime framebuffer access
    Screen clearing
    Pixel-format detection
    Boot diagnostics

The framebuffer becomes the foundation of:

    Writer output
    Logger output
    Panic diagnostics

---

## 6. Writer Initialization

The Writer subsystem is initialized before most other kernel code executes.

Current responsibilities include:

    Text rendering
    Console output
    In-kernel diagnostics

The Writer becomes globally available through:

    WRITER

after initialization.

---

## 7. Logger Initialization

The logging subsystem is brought online during kernel_main().

Current logging targets include:

    Serial output
    Framebuffer output

This provides diagnostics for all later initialization stages.

---

## 8. Transition to kernel_init()

Once basic runtime facilities exist:

    Framebuffer
    Writer
    Logger

control transfers to:

    kernel_init()

This marks the beginning of kernel subsystem initialization.

---

## 9. Responsibilities of kernel_init()

kernel_init() is the central subsystem bootstrap routine.

Current responsibilities include:

    Heap initialization
    FAT32 initialization
    Root directory enumeration
    GDT setup
    TSS setup
    Legacy PIC disable
    IDT setup
    Syscall initialization
    LAPIC mapping
    APIC initialization
    Interrupt enable
    VFS initialization
    File-descriptor initialization
    Optional user-mode startup

Once complete, the kernel is fully operational.

---

## 10. Heap Initialization

The first major task is heap initialization.

Requirements:

    Mapper initialized
    Frame allocator initialized

Current result:

    Dynamic allocation enabled
    Vec available
    Box available

Many higher-level subsystems depend upon a functioning heap.

---

## 11. FAT32 Initialization

After the heap is operational Bulldog initializes FAT32.

Current sequence:

    Create RamDisk
            ↓
    Read BPB
            ↓
    Load FAT table
            ↓
    Register global FAT
            ↓
    Validate filesystem

Initialization information is logged for verification.

This stage provides filesystem access for later ELF loading.

---

## 12. Filesystem Discovery

After FAT initialization:

    Root directory entries

are enumerated and logged.

This confirms:

    FAT image validity
    FAT table correctness
    Root-cluster accessibility
    User program availability

The discovered files become available through the VFS.

---

## 13. GDT and TSS Initialization

Bulldog initializes:

    Global Descriptor Table
    Task State Segment

This provides:

    Ring 0 execution environment
    Ring 3 transition support
    IST stack configuration

This stage must complete before interrupts are enabled.

---

## 14. Legacy PIC Disable

Bulldog disables the legacy PIC:

    PIC1
    PIC2

by masking all IRQ lines.

This prevents legacy interrupt delivery and ensures that interrupt handling occurs through APIC infrastructure.

---

## 15. Interrupt Descriptor Table

The kernel installs the IDT.

Current handlers include:

    CPU exceptions
    Page fault handler
    Double fault handler
    LAPIC timer handler
    Spurious interrupt handler

The IDT must be operational before interrupts are enabled.

---

## 16. Syscall Initialization

The syscall subsystem installs:

    System-call entry mechanism

Current capabilities include:

    Process exit
    File-descriptor operations
    Memory-related services
    User/kernel transitions

The syscall entry point must be installed before user-mode execution.

---

## 17. LAPIC MMIO Mapping

Before APIC initialization:

    LAPIC MMIO

must be mapped into virtual memory.

Current implementation maps:

    Physical:
        0xFEE00000

    Virtual:
        LAPIC_VIRT_BASE

This permits safe APIC access from kernel code.

---

## 18. APIC Initialization

The Local APIC is initialized after MMIO mapping.

Current initialization includes:

    APIC discovery
    APIC enable
    APIC ID reporting
    APIC version reporting
    LAPIC timer configuration
    Spurious interrupt configuration

The APIC timer becomes the foundation of future scheduling work.

---

## 19. Interrupt Enable

Once all required infrastructure exists:

    x86_64::instructions::interrupts::enable()

is executed.

At this point:

    APIC is active
    IDT is loaded
    Interrupt delivery is possible

The kernel can safely receive interrupts.

---

## 20. Virtual File System Initialization

After interrupts are enabled:

    vfs_init()

is executed.

Current responsibilities:

    Filesystem abstraction
    File access services
    User-program loading support

The VFS becomes the primary interface used by user-mode loading.

---

## 21. File Descriptor Initialization

The kernel initializes standard descriptors:

    stdin
    stdout
    stderr

This provides a consistent runtime environment for user processes.

---

## 22. Optional User-Mode Execution

Depending on compile-time features:

    usermode_hello
    usermode_goodbye
    usermode_chain

kernel_init() may immediately execute a user program.

Current supported targets:

    /bin/hello
    /bin/goodbye

User-mode support is operational.

---

## 23. Usermode Execution Path

Current flow:

    Read ELF
            ↓
    Validate ELF
            ↓
    Load Program Headers
            ↓
    Map ELF Segments
            ↓
    Allocate User Stack
            ↓
    Create Process Record
            ↓
    Update Process Metadata
            ↓
    Enter Ring 3

This path forms the foundation of Bulldog's process subsystem.

---

## 24. Process Integration

During usermode startup Bulldog performs:

    create_process()
    update_process_execution()
    set_current_pid()
    mark_running()

The process subsystem therefore participates directly in kernel initialization.

Process lifecycle management is documented separately in:

    processes.md

---

## 25. Usermode Chaining

Bulldog currently supports chained user-process execution.

Current verified sequence:

    /bin/hello
            ↓
    sys_exit()
            ↓
    Cleanup
            ↓
    /bin/goodbye
            ↓
    sys_exit()
            ↓
    Kernel Idle

This mechanism validates:

    Process cleanup
    Frame recycling
    Re-entry into Ring 3
    Process-table management

---

## 26. Completion of kernel_init()

When initialization completes:

    Ok(())

is returned and either:

    User-mode execution begins

or:

    hlt_loop()

is entered,

which eventually becomes the kernel's idle state.

---

## 27. Future Work

Planned future improvements include:

    Scheduler
    Context switching
    Per-process page tables
    Copy-on-write
    Shared memory
    SMP support
    IOAPIC integration
    Advanced process management

These systems will build upon the infrastructure established by kernel_init().

---

## Contributor Notes

- Keep kernel_main() focused on runtime setup.
- Keep kernel_init() focused on subsystem initialization.
- Do not move filesystem logic into kernel_main().
- Do not move framebuffer creation into kernel_init().
- Syscalls must be initialized before user-mode execution.
- APIC setup must occur after LAPIC MMIO mapping.
- Future scheduler work should build on the existing process subsystem.
- The current initialization order is intentional and should be changed carefully.

---

## License

MIT or Apache 2.0 — TBD.