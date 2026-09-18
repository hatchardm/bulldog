# Bulldog Kernel – Privilege Switching

This document describes Bulldog’s privilege switching model, including transitions between
Ring 3 and Ring 0, stack switching, segment selectors, and syscall entry/exit behavior.

It complements:

- `tss.md`
- `stack.md`
- `syscall-entry.md`
- `syscall-implementation.md`
- `interrupts.md`

---

## 1. Overview

Privilege switching occurs when:

- User mode executes `syscall`
- CPU exceptions occur in user mode
- Interrupts occur in user mode

Bulldog uses:

- TSS.rsp0 for syscall stack switching
- IST stacks for critical exceptions
- GDT selectors for segment privilege levels

---

## 2. Ring 3 → Ring 0 (Syscall)

When user mode executes `syscall`, the CPU:

1. Loads kernel CS/SS from `STAR`
2. Loads kernel RIP from `LSTAR`
3. Switches to TSS.rsp0
4. Clears selected RFLAGS bits
5. Begins executing kernel code

This is the fastest privilege switch available on x86_64.

---

## 3. Ring 3 → Ring 0 (Exceptions)

Exceptions occurring in user mode use the IDT:

- CPU loads kernel CS/SS from IDT gate
- CPU switches to IST stack if configured
- CPU pushes an exception frame
- Handler executes in Ring 0

Page faults and double faults use IST stacks.

---

## 4. Ring 3 → Ring 0 (Interrupts)

Interrupts occurring in user mode:

- Use IDT gates
- Switch to IST stack if configured
- Push interrupt frame
- Execute handler in Ring 0

LAPIC timer uses IST1.

---

## 5. Ring 0 → Ring 3 (Sysret)

Syscalls return via:

```
sysretq
```

This restores:

- User RIP  
- User RFLAGS  
- User RCX  
- User R11  
- User stack  

Sysret requires:

- Canonical user RIP  
- Valid user CS  
- Sanitized RFLAGS  

---

## 6. Segment Selectors

Bulldog uses:

- Kernel code segment (Ring 0)
- Kernel data segment (Ring 0)
- User code segment (Ring 3)
- User data segment (Ring 3)

These are defined in the GDT and referenced by:

- `STAR` MSR
- IDT gates

---

## 7. Future Enhancements

- Per‑CPU privilege switching paths (SMP)
- Guard pages for all stacks
- User‑mode exception recovery
- User‑mode signal delivery
- Capability‑based security model

---

## License
MIT or Apache 2.0 — TBD.
