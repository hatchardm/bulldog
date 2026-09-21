# Bulldog Kernel – Task State Segment (TSS)

This document describes Bulldog’s Task State Segment (TSS) architecture, including its role in
privilege switching, interrupt stack tables (IST), and how it integrates with the GDT and
interrupt subsystem.

It complements:

- `stack.md`
- `interrupts.md`
- `interrupts-implementation.md`
- `privilege-switching.md`
- `gdt.md`

---

## 1. Overview

The TSS is a hardware structure used by x86_64 CPUs to manage:

- Privilege‑level stack switching (Ring 3 → Ring 0)
- Interrupt Stack Table (IST) entries
- I/O permission bitmap (unused in Bulldog)
- Reserved fields for hardware task switching (unused in long mode)

Bulldog uses the TSS exclusively for:

- Kernel stack switching during syscalls
- IST stacks for double faults, page faults, and LAPIC timer interrupts

---

## 2. TSS Structure

Bulldog uses the long‑mode TSS layout:

- `rsp0` – kernel stack pointer for Ring 0
- `interrupt_stack_table[0..7]` – IST entries
- `iomap_base` – unused (set to `0xFFFF`)

The TSS is allocated statically and initialized during early boot.

---

## 3. Kernel Stack (RSP0)

When user mode executes `syscall`, the CPU loads:

```
RSP ← TSS.rsp0
```

This becomes the kernel’s entry stack for syscall handling.

Bulldog sets `rsp0` to the main kernel stack provided by the bootloader.

---

## 4. Interrupt Stack Table (IST)

Bulldog uses two IST entries:

| IST Index | Purpose | Backing Stack |
|-----------|---------|---------------|
| **0** | Double fault | `STACK` |
| **1** | LAPIC timer + page fault | `LAPIC_STACK` |

These stacks are defined in `stack.rs` and wired into the TSS in `gdt.rs`.

IST ensures that critical exceptions do not reuse a corrupted kernel stack.

---

## 5. TSS Initialization

During GDT setup:

1. TSS is allocated and zeroed
2. `rsp0` is set to the main kernel stack
3. IST entries are set to the ends of static IST stacks
4. TSS descriptor is added to the GDT
5. `load_tss()` loads the TSS selector

This must occur **before enabling interrupts**.

---

## 6. Integration with Interrupts

The interrupt subsystem uses IST entries for:

- Double fault
- Page fault
- LAPIC timer

Handlers specify IST indices via:

```rust
.set_stack_index(DOUBLE_FAULT_IST_INDEX)
```

This ensures safe stack switching during critical faults.

---

## 7. Future Enhancements

- Per‑CPU TSS (SMP)
- Dedicated syscall IST stack
- Guard pages around IST stacks
- I/O permission bitmap for user‑mode drivers
- TSS dumping tools for debugging

---

## License
MIT or Apache 2.0 — TBD.
