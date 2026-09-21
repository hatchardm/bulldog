# Bulldog Kernel – Interrupt Subsystem

This document describes Bulldog’s interrupt subsystem, including the IDT, exception handling,
interrupt vectors, LAPIC integration, and the architectural rules governing interrupt
delivery. It complements:

- `idt-exceptions.md`
- `apic.md`
- `apic-timer.md`
- `syscall-entry.md`
- `privilege-switching.md`

---

## 1. Overview

The interrupt subsystem provides:

- Exception handling  
- Hardware interrupt routing  
- LAPIC timer integration  
- Spurious interrupt handling  
- Syscall entry vector  
- Deterministic fault reporting  

Bulldog uses the x86_64 interrupt descriptor table (IDT) to route all exceptions and
interrupts to kernel handlers.

---

## 2. Interrupt Descriptor Table (IDT)

The IDT contains 256 entries:

- 0–31: CPU exceptions  
- 32–255: hardware interrupts, LAPIC vectors, syscall vector, custom vectors  

Bulldog allocates a global IDT with interior mutability:

```rust
static IDT: IdtCell = IdtCell(UnsafeCell::new(InterruptDescriptorTable::new()));
```

The IDT is initialized with interrupts disabled to ensure safe mutation.

---

## 3. Exception Handling

Bulldog installs handlers for all architectural exceptions:

- Divide error  
- Debug  
- NMI  
- Breakpoint  
- Overflow  
- Bound range exceeded  
- Invalid opcode  
- Device not available  
- Double fault  
- Invalid TSS  
- Segment not present  
- Stack fault  
- General protection fault  
- Page fault  
- x87 floating point  
- Alignment check  
- Machine check  
- SIMD floating point  
- Virtualization exceptions  
- Security exceptions  

Each handler logs diagnostic information and terminates the kernel.  
User‑mode recovery will be added in a future milestone.

---

## 4. IST (Interrupt Stack Table)

Bulldog uses IST entries for:

- Double fault  
- Page fault  
- LAPIC timer  

This prevents stack corruption during nested faults.

Example:

```rust
idt.double_fault
    .set_handler_fn(double_fault_handler)
    .set_stack_index(DOUBLE_FAULT_IST_INDEX);
```

---

## 5. LAPIC Timer Interrupt

The LAPIC timer uses vector:

```
0x31
```

Handler:

- Increments kernel tick  
- Sends EOI to LAPIC  
- Uses IST for reliability  

---

## 6. Spurious Interrupts

Spurious LAPIC interrupts use vector:

```
0xFF
```

Handler logs and acknowledges the interrupt.

---

## 7. Syscall Vector

The syscall entry vector is reserved and excluded from default handlers:

```rust
i == SYSCALL_VECTOR as usize
```

Syscall entry is documented separately in `syscall-entry.md`.

---

## 8. Default Handlers

Unassigned vectors are routed to a default handler that logs the interrupt.

This ensures no vector is left uninitialized.

---

## 9. IDT Loading

The IDT is loaded via:

```rust
unsafe { idt_ref().load(); }
```

This executes `lidt` and activates the interrupt subsystem.

---

## 10. Future Enhancements

- Per‑process exception recovery  
- Page fault recovery  
- Guard pages  
- User‑mode signals  
- Interrupt statistics  
- Multi‑CPU interrupt routing  

---

## License
MIT or Apache 2.0 — to be determined.

