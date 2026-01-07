# Bulldog Kernel – Interrupts and Exceptions (Current Implementation)

> **Disclaimer:**  
> This document describes Bulldog’s *current* interrupt and exception implementation as it exists in the code today.  
> It does **not** cover future plans for user‑mode, IOAPIC, hardware IRQ remapping, or SMP.  
> Syscalls are handled via a dedicated syscall subsystem, not via `int 0x80`.  
>  
> For related subsystems, see:  
> - `gdt.md` (TSS, IST stacks)  
> - `apic.md` (LAPIC and timer)  
> - `memory.md` (paging, direct map)  
> - `kernel_init.md` (initialization sequence)  
> - `syscall.md` (once implemented)

This document reflects the behavior of `interrupts.rs` as it is implemented today.

---

# 1. Overview

Bulldog’s interrupt subsystem currently provides:

- A statically allocated, global IDT  
- Handlers for all standard CPU exceptions  
- IST‑backed handlers for page faults and double faults  
- A LAPIC timer interrupt on a dedicated vector  
- A spurious interrupt handler  
- Default handlers for all unassigned vectors  
- Explicit skipping of the syscall vector for integration with the syscall subsystem  

The legacy PIC is disabled early in `kernel_init`, and the LAPIC is used as the primary interrupt source.

---

# 2. Interrupt Descriptor Table (IDT)

The IDT is stored in a global cell with interior mutability:

```rust
struct IdtCell(UnsafeCell<InterruptDescriptorTable>);
static IDT: IdtCell = IdtCell(UnsafeCell::new(InterruptDescriptorTable::new()));
```

Access helpers:

- `idt_ref() -> &'static InterruptDescriptorTable`  
- `idt_mut() -> &'static mut InterruptDescriptorTable`  

`init_idt()`:

- Runs inside `interrupts::without_interrupts(...)`  
- Initializes all standard CPU exception entries  
- Installs IST‑backed entries for page fault and double fault  
- Installs LAPIC timer and spurious handlers  
- Installs a few custom debug vectors  
- Fills remaining unset entries with a default handler (excluding syscall vector and some reserved exceptions)  
- Logs selected vectors for debugging  
- Loads the IDT (`lidt`) at the end

This design ensures IDT mutation is safe and deterministic.

---

# 3. Vector Usage (Current Layout)

## 3.1 Key vectors

- **CPU exceptions:** 0x00–0x1F (standard architectural vectors)  
- **LAPIC timer:**  
  ```rust
  pub const LAPIC_TIMER_VECTOR: u8 = 0x31;
  ```  
- **Spurious LAPIC interrupt:**  
  ```rust
  const SPURIOUS_VECTOR: u8 = 0xFF;
  ```  
- **Syscall vector:**  
  Defined in the syscall subsystem as `SYSCALL_VECTOR`.  
  The IDT initialization **skips** installing a default handler for this vector so the syscall subsystem can own it.

## 3.2 Custom debug vectors

Bulldog currently installs diagnostic handlers for:

- Vector 32 → `log_vector_32`  
- Vector 33 → `log_vector_33`  
- Vector 48 → `unhandled_vector_48`  
- Vector 50 → `log_vector_50`  
- Vector 255 → `unhandled_vector_255`  

These are used for testing and debugging.

## 3.3 Default handlers

For all other vectors (0–255), `init_idt()`:

```rust
for i in 0..256 {
    let skip = i == 8
        || (10..=15).contains(&i)
        || (17..=18).contains(&i)
        || (21..=27).contains(&i)
        || (29..=31).contains(&i)
        || i == LAPIC_TIMER_VECTOR as usize
        || i == SYSCALL_VECTOR as usize;

    if skip || idt[i].handler_addr().as_u64() != 0 {
        continue;
    }

    unsafe {
        idt[i].set_handler_fn(default_handler);
    }
}
```

`default_handler` logs `"UNHANDLED INTERRUPT"`.

---

# 4. IST Usage and Critical Exceptions

The IDT integrates with the TSS and IST stacks defined in `gdt.rs` / `stack.rs`.

## 4.1 IST indices

- `DOUBLE_FAULT_IST_INDEX` → IST entry for **double faults**  
- `LAPIC_IST_INDEX` → IST entry for **page faults** and **LAPIC timer**

## 4.2 IST‑backed entries

In `init_idt()`:

```rust
unsafe {
    idt.page_fault
        .set_handler_fn(page_fault_handler)
        .set_stack_index(LAPIC_IST_INDEX);

    idt.double_fault
        .set_handler_fn(double_fault_handler)
        .set_stack_index(DOUBLE_FAULT_IST_INDEX);

    idt[LAPIC_TIMER_VECTOR as usize]
        .set_handler_fn(lapic_timer_handler)
        .set_stack_index(LAPIC_IST_INDEX);

    idt[SPURIOUS_VECTOR as usize].set_handler_fn(spurious_handler);
}
```

## 4.3 Exception behavior

Handlers for exceptions such as:

- Divide error  
- Debug  
- NMI  
- Page fault  
- Double fault  
- Invalid opcode  
- General protection fault  
- Alignment check  
- Machine check  
- SIMD floating point  
- Virtualization  
- Security  

…all:

- Log the exception name  
- Log the `InterruptStackFrame`  
- For faults with error codes, log the error code  
- `panic!` for all fatal conditions  

`page_fault_handler` additionally logs CR2 (faulting address) and the `PageFaultErrorCode`.

There is no recovery path yet; all serious exceptions are fatal, which is appropriate for early kernel development.

---

# 5. LAPIC Timer and Spurious Interrupts

## 5.1 LAPIC timer handler

The LAPIC timer interrupt is handled by:

```rust
extern "x86-interrupt" fn lapic_timer_handler(_stack_frame: InterruptStackFrame) {
    tick();
    send_eoi();
}
```

Behavior:

- `tick()` updates the kernel time/tick subsystem  
- `send_eoi()` sends EOI to the LAPIC  

The handler:

- Does **not** log on every tick (to avoid log spam)  
- Does **not** panic  
- Runs on IST1 (`LAPIC_IST_INDEX`) for robustness

## 5.2 Spurious interrupt handler

Spurious interrupts use vector `SPURIOUS_VECTOR` (0xFF):

```rust
extern "x86-interrupt" fn spurious_handler(_stack_frame: InterruptStackFrame) {
    error!("SPURIOUS INTERRUPT");
    send_eoi();
}
```

Behavior:

- Logs the spurious interrupt  
- Sends EOI to clear the LAPIC state  

---

# 6. Syscall Vector Integration

Bulldog’s syscalls are **not** implemented via `int 0x80`.  
Instead:

- The syscall subsystem defines a `SYSCALL_VECTOR`  
- `init_idt()` explicitly skips installing a default handler for `SYSCALL_VECTOR`:

```rust
|| i == SYSCALL_VECTOR as usize
```

This ensures:

- The syscall subsystem can install its own handler (or use SYSCALL/SYSRET)  
- The interrupt layer does not interfere with syscall behavior  

The exact syscall entry mechanics are documented in `syscall.md` (once completed).

---

# 7. Logging and Debugging

The interrupt subsystem logs:

- Exception type and name  
- `InterruptStackFrame` (registers and CS/RIP/RSP/RFLAGS)  
- Error codes where applicable (e.g., page fault, GPF, alignment check)  
- Faulting address for page faults (CR2)  
- Selected IDT entries’ handler addresses during initialization  
- Spurious and unhandled interrupts  

This logging is critical for early bring‑up and debugging.

---

# 8. Contributor Notes

- `init_idt()` must be called before enabling interrupts  
- IST indices must match the TSS configuration in `gdt.rs`  
- The LAPIC timer handler **must** call `send_eoi()`  
- Do not install handlers on `SYSCALL_VECTOR` in `interrupts.rs`; the syscall subsystem owns it  
- Avoid heavy work in interrupt handlers; keep them short and deterministic  
- When adding new vectors, document them here and maintain clear separation of concerns  

---

# License

MIT or Apache 2.0 — TBD.
