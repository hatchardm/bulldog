# Bulldog Kernel – Architecture Overview

This document provides a high‑level overview of Bulldog’s internal architecture.  
It serves as the entry point for subsystem documentation, including privilege switching,
syscalls, and the syscall harness.

Bulldog is a custom operating system kernel written in Rust for the `x86_64-bulldog`
architecture. Its design emphasizes clarity, safety, and contributor accessibility.

---

## Architecture Components

Bulldog’s architecture is organized into the following major subsystems:

- Memory management and paging  
- Interrupt handling and vector hygiene  
- GDT/TSS initialization  
- Privilege switching (Ring 0 ↔ Ring 3)  
- Syscall interface and dispatcher  
- Syscall table and numbering  
- Syscall test harness  
- Early user‑mode execution  

Each subsystem has its own dedicated documentation.

---

## Subsystem Documentation

### Privilege Switching

Mechanics of entering and exiting user mode, including:

- GDT/TSS configuration  
- Stack switching  
- IDT gate permissions  
- Return‑path invariants  

See:  
`../privilege-switching.md`

---

### Syscall Development

Overview of Bulldog’s syscall model, including:

- Syscall lifecycle  
- Dispatcher design  
- Calling conventions  
- Contributor tasks and hygiene  

See:  
`../syscall.md`

---

### Syscall Table

Defines syscall numbering, handler mapping, and contributor workflow for adding new syscalls.

See:  
`../syscall-table.md`

---

### Syscall Harness

Describes the test harness used to validate syscall entry, dispatch, register preservation,
and return‑path correctness.

Includes:

- User‑mode test program  
- Logging pipeline  
- Error injection tests  
- Register preservation checks  

See:  
`../syscall-harness-guide.md`

---

## Roadmap Context

Bulldog’s architecture evolves through milestone branches:

- `feature/pic8259` – Legacy PIC baseline  
- `feature/apic` – APIC milestone  
- `feature/syscall` – Privilege switching + syscall infrastructure  

Future milestones will expand:

- User‑mode execution  
- Process scheduling  
- Virtual memory enhancements  
- Filesystem interfaces  

---

## Contributor Notes

- Keep subsystem documentation updated as features evolve  
- Maintain consistent formatting across all architecture documents  
- Document unsafe blocks with justification  
- Ensure logs and test harness output remain deterministic  

---

## License

MIT or Apache 2.0 — to be determined.
