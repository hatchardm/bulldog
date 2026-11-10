## Bulldog Kernel – Syscall Entry Path

Reviewed: November 2026

This document describes Bulldog's current syscall entry mechanism.

It reflects the implementation used by:

    syscall.rs
    dispatcher.rs
    privilege-switching.md

and the syscall harness.

This document describes the current implementation rather than future syscall designs.

---

## 1. Overview

Syscalls provide the only legal mechanism for user-mode code to enter the kernel.

Bulldog currently uses:

    Interrupt-based syscall entry

rather than:

    syscall/sysret MSR-based entry

User-mode software invokes a dedicated syscall interrupt vector that transitions execution from:

    Ring 3
        ↓
    Ring 0

under kernel control.

---

## 2. Syscall Vector

Bulldog reserves:

    SYSCALL_VECTOR

for syscall entry.

Current configuration:

    Vector = 0x80
    DPL    = 3

Properties:

    User callable
    Installed by syscall subsystem
    Not installed by interrupts::init_idt()
    Reserved exclusively for syscalls

This vector forms the entry point into the syscall subsystem.

---

## 3. Why A Dedicated Vector

Using a dedicated vector provides:

    Controlled kernel entry
    Explicit privilege transitions
    Simple debugging
    Deterministic behaviour

It also separates syscall handling from:

    Exceptions
    LAPIC interrupts
    Spurious interrupts

---

## 4. Entry Path

Current syscall flow:

    User Program
            ↓
    int 0x80
            ↓
    IDT Gate
            ↓
    CPU privilege transition
            ↓
    TSS.rsp0 activation
            ↓
    Kernel entry stub
            ↓
    syscall_entry()
            ↓
    syscall_dispatch()
            ↓
    Syscall Handler

This is the current Ring 3 → Ring 0 path.

---

## 5. Privilege Transition

The syscall gate is configured with:

    DPL = 3

allowing invocation from user mode.

CPU actions:

    Validate IDT gate
            ↓
    Switch to Ring 0
            ↓
    Load RSP0 from TSS
            ↓
    Push interrupt frame
            ↓
    Execute kernel entry path

The CPU therefore never executes kernel code using the user stack.

---

## 6. Stack Switching

Stack switching is provided by:

    TSS.rsp0

Backing stack:

    KERNEL_STACK

Purpose:

    Provide trusted kernel execution stack
    Isolate kernel execution from user memory
    Support reliable syscall execution

All syscall entry begins on this stack.

---

## 7. Entry Stub Responsibilities

The syscall entry stub is responsible for:

    Preserving required registers
    Capturing user context
    Building syscall state
    Calling syscall_entry()

The stub should remain:

    Minimal
    Deterministic
    Easy to audit

---

## 8. syscall_entry()

syscall_entry() provides the Rust entry point into the syscall subsystem.

Responsibilities:

    Extract syscall number
    Extract syscall arguments
    Invoke dispatcher
    Return syscall result

This function acts as the bridge between low-level entry code and kernel syscall handlers.

---

## 9. Dispatcher Integration

After entry:

    syscall_dispatch()

performs:

    Syscall lookup
    Handler dispatch
    Error handling

Typical flow:

    Syscall Number
            ↓
    Lookup Table
            ↓
      Found?
      /    \
    Yes     No
     |       |
     ↓       ↓
 Handler   ENOSYS
     |
     ↓
 Return

---

## 10. Return Path

After syscall completion:

    Return value → RAX
            ↓
    Restore user context
            ↓
    Restore privilege state
            ↓
    Return to Ring 3

Execution resumes at the instruction following:

    int 0x80

in the calling user process.

---

## 11. Current Syscall Flow Example

Example:

    /bin/hello
            ↓
    int 0x80
            ↓
    sys_write()
            ↓
    Return
            ↓
    Continue User Execution

The syscall harness validates this path extensively.

---

## 12. Error Handling

Invalid syscall numbers return:

    ENOSYS

rather than causing exceptions.

Example:

    Unknown syscall 999
            ↓
    ENOSYS

This keeps syscall behaviour deterministic.

---

## 13. Current Architecture Summary

Implemented:

    Ring 3 syscall entry
    IDT syscall vector
    DPL=3 gate
    TSS stack switching
    Dispatcher integration
    Return-value handling
    Syscall harness validation

Verified through:

    sys_write
    sys_read
    sys_open
    sys_close
    sys_alloc
    sys_free
    sys_exit

---

## 14. Future Enhancements

Planned improvements:

    Per-process kernel stacks
    Per-CPU kernel stacks
    SMP support
    Syscall tracing
    Syscall filtering
    User-pointer validation
    Advanced syscall dispatch

These features are not currently implemented.

---

## Contributor Notes

- SYSCALL_VECTOR is reserved exclusively for syscall entry.
- The syscall vector must have DPL=3.
- TSS.rsp0 must always remain valid.
- Keep syscall entry deterministic.
- Keep dispatcher behavior synchronized with dispatcher.md.
- Review this document whenever syscall entry changes.

---

## License

MIT or Apache 2.0 — TBD.
