## Bulldog Kernel – CPU Initialization Architecture

Reviewed: November 2026

This document describes Bulldog's current CPU initialization architecture.

It covers:

    GDT initialization
    TSS initialization
    Ring 3 support
    IST stack configuration
    IDT installation
    Exception handling
    LAPIC initialization
    Timer interrupts
    Interrupt enablement

This document reflects the current implementation found in:

    kernel/src/gdt.rs
    kernel/src/interrupts.rs
    kernel/src/apic.rs
    kernel/src/lib.rs

It documents the current architecture rather than future plans.

---

## 1. Overview

Bulldog enters the kernel after the bootloader and kernel-bin have already established a functioning execution environment.

Current startup path:

    Bootloader
            ↓
    kernel-bin
            ↓
    kernel_main()
            ↓
    kernel_init()
            ↓
    GDT / TSS
            ↓
    PIC Disable
            ↓
    IDT
            ↓
    Syscalls
            ↓
    LAPIC Mapping
            ↓
    APIC Initialization
            ↓
    Interrupt Enable

CPU initialization occurs primarily during:

    kernel_init()

before interrupts are enabled.

---

## 2. CPU State At Kernel Entry

When Bulldog enters the kernel:

    Long mode is active
    Paging is active
    HHDM is active
    CR3 is already loaded
    Higher-half execution is active

The bootloader has already:

    Created page tables
    Created HHDM mappings
    Enabled paging
    Loaded CR3

The kernel does not perform a higher-half transition.

---

## 3. CPU Initialization Goals

Before interrupts are enabled Bulldog must construct:

    GDT
    TSS
    RSP0 stack
    IST stacks
    IDT
    LAPIC mappings
    APIC configuration

These structures form the foundation of:

    Exception handling
    Interrupt handling
    Syscalls
    Ring 3 execution
    Future scheduling

---

## 4. Global Descriptor Table

Bulldog initializes a dedicated Global Descriptor Table during:

    gdt::init()

Current descriptors include:

    Kernel Code Segment
    Kernel Data Segment
    User Code Segment
    User Data Segment
    TSS Descriptor

After creation the GDT is loaded and segment registers are updated.

Current registers configured:

    CS
    DS
    ES
    SS

The TSS is loaded immediately afterward.

---

## 5. Task State Segment

Bulldog constructs a Task State Segment during CPU initialization.

The TSS provides:

    Ring 3 → Ring 0 stack transitions
    Interrupt Stack Tables
    Exception recovery stacks

Current TSS configuration includes:

    RSP0
    Double-fault IST
    LAPIC/Page-fault IST

The TSS is activated using:

    load_tss()

after GDT installation.

---

## 6. Privilege Stack (RSP0)

Bulldog configures:

    privilege_stack_table[0]

for Ring 3 to Ring 0 transitions.

Backing stack:

    KERNEL_STACK

Current stack size:

    128 KiB

This stack becomes active when:

    int 0x80
    Exceptions
    Interrupts from Ring 3

cause a transition into kernel mode.

This stack is separate from IST stacks.

---

## 7. Interrupt Stack Tables

Bulldog currently maintains dedicated IST entries.

### IST0

Purpose:

    Double-fault handling

Reason:

    Survive stack-corruption scenarios

Current constant:

    DOUBLE_FAULT_IST_INDEX = 0

Backing stack:

    STACK

Size:

    128 KiB

---

### IST1

Purpose:

    LAPIC timer interrupts
    Page-fault handling

Current constant:

    LAPIC_IST_INDEX = 1

Backing stack:

    LAPIC_STACK

Size:

    128 KiB

Using a dedicated stack improves robustness during fault handling and interrupt processing.

---

## 8. Interrupt Descriptor Table

The Interrupt Descriptor Table is initialized by:

    interrupts::init_idt()

The IDT is populated before interrupts are enabled.

Current categories include:

    CPU exceptions
    Page faults
    Double faults
    LAPIC timer interrupts
    Spurious interrupts
    Default interrupt handlers

The completed table is loaded using:

    lidt

during initialization.

---

## 9. Exception Handling

Bulldog currently installs handlers for:

    Divide Error
    Debug
    Non-Maskable Interrupt
    Breakpoint
    Overflow
    Bound Range Exceeded
    Invalid Opcode
    Device Not Available
    Invalid TSS
    Segment Not Present
    Stack Segment Fault
    General Protection Fault
    X87 Floating Point
    Alignment Check
    Machine Check
    SIMD Floating Point
    Virtualization
    Control Protection Exception
    Hypervisor Injection Exception
    Security Exception
    Page Fault
    Double Fault

Most handlers:

    Log diagnostics
    Dump CPU state
    Panic

This behaviour is intentional during early OS development.

---

## 10. Page Fault Handling

Page-fault handling receives:

    Fault address (CR2)
    Error code
    Interrupt stack frame

Current diagnostics include:

    Fault address
    Error code
    CPU state dump

Page faults execute on the LAPIC/Page-Fault IST stack.

This improves reliability if the current execution stack is corrupted.

---

## 11. Double-Fault Handling

Double faults execute on a dedicated IST stack.

Current behaviour:

    Log diagnostic information
    Dump interrupt frame
    Panic

The dedicated stack is specifically intended to survive stack corruption scenarios.

---

## 12. LAPIC Timer Interrupts

Bulldog currently uses:

    Vector 0x31

for the Local APIC timer.

Current responsibilities:

    Kernel tick generation
    Timekeeping

Current timer handler:

    lapic_timer_handler()

The scheduler does not yet use these interrupts.

---

## 13. Spurious Interrupt Handling

Bulldog installs a spurious interrupt handler at:

    Vector 0xFF

Current responsibilities:

    Log interrupt occurrence
    Send EOI

This vector is also used during APIC enablement.

---

## 14. Legacy PIC Disable

Before APIC initialization Bulldog disables:

    PIC1
    PIC2

by masking all legacy IRQ lines.

This prevents interrupt delivery through the legacy PIC.

All interrupt handling thereafter is APIC based.

---

## 15. LAPIC MMIO Mapping

Before APIC initialization the Local APIC MMIO region must be mapped.

Current mapping target:

    LAPIC_VIRT_BASE

Current LAPIC physical address:

    0xFEE00000

Flags:

    PRESENT
    WRITABLE
    NO_EXECUTE

This mapping occurs before:

    setup_apic()

is called.

---

## 16. APIC Initialization

The Local APIC is configured during:

    setup_apic()

Current initialization includes:

    LAPIC MSR verification
    APIC enable verification
    LAPIC ID reporting
    LAPIC version reporting
    SVR configuration
   