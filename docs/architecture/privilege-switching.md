## Bulldog Kernel – Privilege Switching

Reviewed: November 2026

This document describes Bulldog's current privilege-switching architecture.

It covers:

    Ring 3 execution
    Ring 0 execution
    User-to-kernel transitions
    RSP0 stack switching
    GDT integration
    TSS integration
    Interrupt-based privilege transitions

It reflects the implementation found in:

    gdt.rs
    interrupts.rs
    syscall.rs
    usermode.rs

This document describes the current implementation rather than historical or planned designs.

---

## 1. Overview

Bulldog currently supports execution in:

    Ring 0 (Kernel)
    Ring 3 (User Mode)

Privilege transitions occur when:

    User programs invoke syscalls
    User-mode exceptions occur
    User-mode interrupts occur

The CPU automatically transitions into kernel mode using information stored in:

    GDT
    TSS
    IDT

---

## 2. Current Execution Model

Kernel code executes in:

    Ring 0

User programs execute in:

    Ring 3

Examples:

    /bin/hello
    /bin/goodbye

Current verified execution path:

    Kernel
            ↓
    Ring 3 Entry
            ↓
    User Program
            ↓
    Syscall
            ↓
    Ring 0
            ↓
    Kernel Handler

---

## 3. GDT Integration

Bulldog's GDT currently contains:

    Kernel Code Segment
    Kernel Data Segment
    User Code Segment
    User Data Segment
    TSS Descriptor

These descriptors allow the CPU to enforce privilege separation.

Current selectors include:

    code_selector
    data_selector
    user_code_selector
    user_data_selector

Ring transitions depend on these descriptors.

---

## 4. Task State Segment (TSS)

Bulldog uses a Task State Segment to support privilege transitions.

Responsibilities:

    RSP0 configuration
    IST configuration
    Exception stack selection

The TSS remains active for the lifetime of the kernel.

---

## 5. RSP0 Privilege Stack

Bulldog configures:

    privilege_stack_table[0]

inside the TSS.

Backing stack:

    KERNEL_STACK

Current size:

    128 KiB

Purpose:

    Ring 3 → Ring 0 transitions

When the CPU enters kernel mode from user mode it automatically switches to this stack.

This guarantees that kernel code does not execute on user-controlled stacks.

---

## 6. Ring 3 → Ring 0 (Syscalls)

Current syscall transitions enter Ring 0 through the syscall subsystem.

The transition relies upon:

    User GDT selectors
    Kernel GDT selectors
    TSS.rsp0
    IDT/syscall entry logic

Typical flow:

    User Program
            ↓
    System Call
            ↓
    CPU switches privilege level
            ↓
    RSP0 activated
            ↓
    Kernel Handler

This provides controlled entry into privileged code.

---

## 7. Ring 3 → Ring 0 (Exceptions)

Exceptions occurring in user mode cause an automatic privilege transition.

CPU actions:

    Load kernel privilege level
            ↓
    Switch to kernel stack
            ↓
    Push exception frame
            ↓
    Execute exception handler

Current exception examples:

    Page Fault
    General Protection Fault
    Invalid Opcode
    Double Fault

---

## 8. Ring 3 → Ring 0 (Interrupts)

Interrupts occurring during user execution behave similarly.

CPU actions:

    Switch privilege level
            ↓
    Switch stack
            ↓
    Push interrupt frame
            ↓
    Execute handler

Current example:

    LAPIC Timer Interrupt

The timer interrupt executes using:

    IST1

for additional reliability.

---

## 9. Interrupt Stack Table (IST)

Bulldog currently uses dedicated exception stacks.

### IST0

Used for:

    Double Fault

### IST1

Used for:

    Page Fault
    LAPIC Timer

Dedicated stacks improve fault recovery and interrupt robustness.

---

## 10. Ring 0 → Ring 3

After kernel work completes, execution returns to user mode.

Current restoration includes:

    User instruction pointer
    User stack pointer
    User segment selectors
    User processor state

Successful return resumes execution of the user process.

---

## 11. Current Security Properties

The current design provides:

    Hardware-enforced privilege separation
    Distinct user and kernel segments
    Dedicated kernel entry stack
    Dedicated IST stacks
    Protected kernel memory

User-space code cannot directly execute privileged operations.

---

## 12. Current Limitations

Not yet implemented:

    Per-process page tables
    User-mode exception recovery
    User-mode signal delivery
    SMP privilege-switch support
    Per-CPU TSS structures

Privilege transitions currently operate within the shared address-space model.

---

## 13. Future Enhancements

Planned future work:

    Per-process address spaces
    Per-CPU TSS structures
    SMP support
    User-mode signal delivery
    Advanced syscall infrastructure
    Enhanced fault recovery

---

## Contributor Notes

- RSP0 must always remain valid.
- User and kernel selectors must remain synchronized with gdt.rs.
- IST assignments must remain synchronized with interrupts.rs.
- Privilege-switching changes should be reviewed alongside gdt.md and syscall documentation.
- Ring 3 support depends directly on correct GDT and TSS configuration.

---

## License

MIT or Apache 2.0 — TBD.