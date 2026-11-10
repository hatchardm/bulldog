## Bulldog Kernel – Global Descriptor Table (GDT) and Task State Segment (TSS)

Reviewed: November 2026

This document describes Bulldog's current implementation of the Global Descriptor Table (GDT), Task State Segment (TSS), privilege stacks, and Interrupt Stack Table (IST).

It reflects the implementation found in:

    kernel/src/gdt.rs
    kernel/src/stack.rs
    kernel_init()

This document describes the current implementation rather than future plans.

---

## 1. Overview

Bulldog uses a GDT configuration suitable for a modern x86_64 kernel with Ring 3 support.

Current descriptors include:

    Kernel Code Segment
    Kernel Data Segment
    User Code Segment
    User Data Segment
    TSS Descriptor

The TSS provides:

    Ring 3 → Ring 0 transitions
    Privilege stacks
    Dedicated IST stacks

Current IST assignments:

    IST0 → Double Fault
    IST1 → LAPIC Timer and Page Fault

The corresponding stacks are statically allocated in higher-half kernel memory.

User-mode support is currently implemented and uses dedicated Ring 3 descriptors.

---

## 2. GDT Structure

The GDT is created in:

    kernel/src/gdt.rs

using:

    GlobalDescriptorTable

Current entries:

    Descriptor::kernel_code_segment()
    Descriptor::kernel_data_segment()
    Descriptor::user_code_segment()
    Descriptor::user_data_segment()
    Descriptor::tss_segment()

The GDT is initialized once during kernel startup.

---

## 3. Segment Selectors

Bulldog stores segment selectors in:

    Selectors

Current selectors:

    code_selector
    data_selector
    tss_selector
    user_code_selector
    user_data_selector

These selectors are later used to configure:

    CS
    DS
    ES
    SS
    TSS
    Ring 3 entry

---

## 4. Task State Segment (TSS)

The Task State Segment is created during:

    gdt::init()

Responsibilities:

    IST configuration
    Privilege-stack configuration
    Ring transition support

The TSS is leaked into permanently allocated memory and remains valid for the lifetime of the kernel.

After creation it is loaded using:

    load_tss()

---

## 5. Privilege Stack (RSP0)

Bulldog configures:

    privilege_stack_table[0]

inside the TSS.

Purpose:

    Ring 3 → Ring 0 transitions

Backing stack:

    KERNEL_STACK

Current size:

    128 KiB

The CPU switches to this stack when:

    System calls
    User-mode exceptions
    User-mode interrupts

cause entry into kernel mode.

This stack is separate from the IST mechanism.

---

## 6. Interrupt Stack Table Layout

Current IST assignments:

    IST0 → Double Fault
    IST1 → LAPIC Timer + Page Fault

Constants:

    DOUBLE_FAULT_IST_INDEX = 0
    LAPIC_IST_INDEX        = 1

These assignments are established during TSS creation.

---

## 7. IST Stack Properties

Current stack characteristics:

    128 KiB per stack
    16-byte aligned
    Statically allocated
    Higher-half virtual addresses

Stack pointers are initialized using:

    stack_base + STACK_SIZE

because x86_64 stacks grow downward.

Current stacks:

    STACK
    LAPIC_STACK
    KERNEL_STACK

Guard pages are not currently implemented.

---

## 8. IST0 – Double Fault Handler

Purpose:

    Double-fault recovery

Typical fault causes:

    Stack corruption
    Fault recursion
    Invalid exception-handling state

Using a dedicated stack allows the CPU to continue operating even if the current stack is unusable.

Current backing stack:

    STACK

---

## 9. IST1 – LAPIC Timer and Page Faults

Purpose:

    LAPIC timer interrupts
    Page faults

Current backing stack:

    LAPIC_STACK

Benefits:

    Improved fault isolation
    Reliable timer handling
    Reduced dependency on the interrupted execution stack

This arrangement improves overall kernel robustness.

---

## 10. GDT Initialization Flow

Current initialization sequence:

    Create TSS
            ↓
    Configure IST stacks
            ↓
    Configure RSP0
            ↓
    Create GDT
            ↓
    Add kernel descriptors
            ↓
    Add user descriptors
            ↓
    Add TSS descriptor
            ↓
    Load GDT
            ↓
    Load CS
            ↓
    Load DS/ES/SS
            ↓
    Load TSS
            ↓
    Store selectors

This occurs during:

    kernel_init()

before interrupts are enabled.

---

## 11. Ring 3 Support

Bulldog currently supports Ring 3 execution.

The GDT provides:

    User Code Segment
    User Data Segment

Accessors:

    user_code_selector()
    user_data_selector()

These selectors are used during user-mode entry and privilege transitions.

Current verified programs:

    /bin/hello
    /bin/goodbye

Ring 3 support is operational.

---

## 12. Interaction With Interrupts

The interrupt subsystem depends on:

    Loaded GDT
    Loaded TSS
    Valid IST entries

Current relationships:

    Double Fault → IST0
    Page Fault   → IST1
    LAPIC Timer  → IST1

Interrupt handling must not occur before GDT/TSS initialization completes.

---

## 13. Interaction With Memory Management

Current memory requirements:

    GDT must remain mapped
    TSS must remain mapped
    IST stacks must remain mapped
    RSP0 stack must remain mapped

The memory subsystem therefore treats these structures as permanent kernel allocations.

Guard pages remain future work.

---

## 14. Interaction With Syscalls

Current system-call support relies on:

    User code selector
    User data selector
    RSP0 privilege stack

When Ring 3 code invokes:

    int 0x80

the CPU transitions into kernel mode using the stack information contained within the TSS.

User-mode execution therefore depends directly on correct GDT and TSS configuration.

---

## 15. Current Architecture Summary

Implemented:

    Kernel code descriptor
    Kernel data descriptor
    User code descriptor
    User data descriptor
    TSS descriptor
    RSP0 privilege stack
    Double-fault IST
    LAPIC/Page-fault IST
    Ring 3 support
    TSS loading

This architecture supports all currently verified user-mode functionality.

---

## 16. Future Work

Planned improvements:

    Guard pages around IST stacks
    Additional IST entries
    Per-CPU GDT structures
    Per-CPU TSS structures
    SMP support
    IOAPIC integration

These features are not currently implemented.

---

## Contributor Notes

- GDT initialization must occur before interrupts are enabled.
- TSS loading must occur after GDT loading.
- RSP0 must remain valid for Ring 3 transitions.
- Double-fault handling depends on IST0.
- LAPIC timer and page-fault handling depend on IST1.
- Stack sizes and IST indices should remain synchronized with gdt.rs.
- Review this document whenever GDT or TSS structures change.

---

## License

MIT or Apache 2.0 — TBD.