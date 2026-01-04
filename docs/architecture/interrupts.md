# Bulldog Kernel – Interrupts and Exception Handling

This document describes Bulldog’s interrupt architecture, including the IDT, exception
handling, APIC configuration, vector hygiene, and early interrupt setup. It provides
contributors with a clear understanding of how interrupts flow through the system and how
they interact with privilege switching and syscalls.

Bulldog targets the `x86_64-bulldog` architecture and uses the APIC model for interrupt
delivery.

---

## Overview

Bulldog’s interrupt subsystem is built around the following principles:

- Deterministic vector layout  
- Clean separation between exceptions, interrupts, and syscalls  
- APIC‑based interrupt delivery  
- Per‑CPU LAPIC timer configuration  
- Safe entry/exit paths with stack switching  
- Contributor‑friendly logging and debugging  

Interrupts are central to privilege switching, syscalls, scheduling, and future process
management.

---

## Interrupt Descriptor Table (IDT)

The IDT defines the entry points for:

- CPU exceptions  
- Hardware interrupts  
- Software interrupts (including syscalls)  

Each entry contains:

- Handler address  
- Segment selector  
- Gate type (interrupt/trap)  
- Descriptor privilege level (DPL)  
- IST index (optional)  

Bulldog uses interrupt gates for all entries to ensure `RFLAGS.IF` is cleared on entry.

---

## Vector Layout

Bulldog maintains a clean, predictable vector layout:

```
0x00–0x1F  → CPU exceptions
0x20–0x2F  → Hardware IRQs (remapped PIC legacy range)
0x30–0x3F  → LAPIC timer and APIC-specific interrupts
0x80       → Primary syscall vector (INT 0x80)
0x81–0x8F  → Reserved for future syscalls
0x90–0xFF  → Reserved / future expansion
```

This layout ensures:

- Exceptions remain at their architectural vector numbers  
- Hardware IRQs are remapped away from 0x00–0x1F  
- Syscalls occupy a dedicated, non‑overlapping range  
- Future expansion is predictable  

---

## Exception Handling

Exceptions include:

- Divide‑by‑zero  
- Page faults  
- General protection faults  
- Invalid opcode  
- Double faults  

Bulldog’s exception handlers:

- Log the exception type  
- Log relevant registers  
- Halt the CPU for fatal conditions  
- Use IST entries for critical exceptions (e.g., double fault)  

Example exception flow:

```
CPU exception
    ↓
IDT entry (DPL=0)
    ↓
Kernel stack (via TSS)
    ↓
Exception handler
    ↓
Log + halt or recover
```

---

## APIC and LAPIC

Bulldog uses the APIC model for interrupt delivery:

- IOAPIC handles external hardware interrupts  
- LAPIC handles per‑CPU interrupts, including the timer  

### LAPIC Timer

The LAPIC timer is used for:

- Preemption (future)  
- Timekeeping  
- Scheduling (future)  

Configuration steps:

1. Map LAPIC registers  
2. Set divide configuration  
3. Load initial count  
4. Enable timer interrupt vector  

---

## Interrupt Entry Path

When an interrupt occurs:

1. CPU pushes `RIP`, `CS`, `RFLAGS`  
2. If privilege level changes, CPU pushes `SS` and `RSP`  
3. CPU loads kernel stack from TSS  
4. IDT entry transfers control to the handler  
5. Handler saves general‑purpose registers  
6. Handler processes the interrupt  
7. Registers are restored  
8. `iretq` returns to the interrupted context  

Diagram:

```
User Mode (Ring 3)
    |
    | interrupt / exception / int 0x80
    v
CPU pushes frame
CPU loads kernel RSP from TSS
    |
    v
Kernel Mode (Ring 0)
    |
    | handler
    v
iretq
    |
    v
User Mode (Ring 3)
```

This path is shared with syscalls, which use vector `0x80`.

---

## Interrupt Stack Table (IST)

Bulldog reserves IST entries for:

- Double fault  
- Non‑maskable interrupt (NMI)  
- Machine check (future)  

IST ensures these critical exceptions have a known‑good stack even if the main kernel stack
is corrupted.

---

## Hardware IRQ Handling

Bulldog remaps legacy PIC IRQs to APIC vectors:

```
IRQ0 → 0x20  (timer)
IRQ1 → 0x21  (keyboard)
...
IRQ15 → 0x2F
```

IRQ handlers:

- Acknowledge the LAPIC/IOAPIC  
- Perform minimal work  
- Defer heavy processing to future scheduling subsystems  

---

## Syscall Interaction

Syscalls use vector `0x80` with:

- DPL = 3 (user‑callable)  
- Interrupt gate (clears IF)  
- Kernel stack switching via TSS  

Syscall entry shares the same interrupt path as hardware IRQs, ensuring consistent behavior.

See:  
`../syscall.md`  
`../privilege-switching.md`  
`../syscall-harness-guide.md`

---

## Logging and Debugging

Interrupt handlers log:

- Vector number  
- Error code (if present)  
- CS, SS, RSP, RFLAGS  
- Register snapshot (optional)  

This logging is essential during early development and debugging.

---

## Contributor Notes

- Maintain vector hygiene when adding new handlers  
- Ensure all IDT entries use correct DPL and gate types  
- Validate IST usage for critical exceptions  
- Keep logs deterministic for debugging and test harness integration  
- Avoid long‑running work inside interrupt handlers  

---

## License

MIT or Apache 2.0 — to be determined.
