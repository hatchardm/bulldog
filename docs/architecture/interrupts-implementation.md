# Bulldog Kernel – Interrupts & Exceptions (Implementation Details)

This document describes Bulldog’s *current* interrupt and exception implementation exactly as
it exists in `kernel/src/interrupts.rs`. It is an implementation‑level reference intended for
contributors working directly with interrupt handling, IDT configuration, LAPIC integration,
and exception behavior.

For architectural overview, see:

- `interrupts.md` (subsystem design)
- `idt-exceptions.md` (exception model)
- `lapic.md` (APIC/LAPIC architecture)
- `apic-timer.md` (timer subsystem)
- `gdt.md` (TSS + IST stacks)
- `syscall-entry.md` (syscall vector)

---

## 1. Global IDT Structure

Bulldog uses a statically allocated IDT with interior mutability:

```rust
struct IdtCell(UnsafeCell<InterruptDescriptorTable>);
static IDT: IdtCell = IdtCell(UnsafeCell::new(InterruptDescriptorTable::new()));
```

Two helpers provide safe access:

- `idt_ref()` → immutable reference  
- `idt_mut()` → mutable reference (only used with interrupts disabled)

The IDT is mutated inside:

```rust
interrupts::without_interrupts(|| { ... })
```

This ensures safe initialization before interrupts are enabled.

---

## 2. IDT Initialization (`init_idt`)

`init_idt()` performs the following steps:

### 2.1 Install all standard CPU exception handlers

Handlers are installed for:

- Divide error  
- Debug  
- NMI  
- Breakpoint  
- Overflow  
- Bound range exceeded  
- Invalid opcode  
- Device not available  
- Invalid TSS  
- Segment not present  
- Stack fault  
- General protection fault  
- Page fault  
- x87 floating point  
- Alignment check  
- Machine check  
- SIMD floating point  
- Virtualization  
- Control protection  
- Hypervisor injection  
- Security exception  

Each handler logs the exception and panics.

### 2.2 Install IST‑backed handlers

IST stacks are defined in the TSS (`gdt.rs`):

- `DOUBLE_FAULT_IST_INDEX`  
- `LAPIC_IST_INDEX`

IST is used for:

```rust
idt.page_fault.set_stack_index(LAPIC_IST_INDEX);
idt.double_fault.set_stack_index(DOUBLE_FAULT_IST_INDEX);
idt[LAPIC_TIMER_VECTOR].set_stack_index(LAPIC_IST_INDEX);
```

This prevents stack corruption during nested faults.

### 2.3 Install LAPIC timer and spurious handlers

- LAPIC timer → vector `0x31`
- Spurious interrupt → vector `0xFF`

### 2.4 Install custom debug/test vectors

Current debug vectors:

- 32 → `log_vector_32`
- 33 → `log_vector_33`
- 48 → `unhandled_vector_48`
- 50 → `log_vector_50`
- 255 → `unhandled_vector_255`

### 2.5 Install default handlers for all remaining vectors

A fallback loop assigns `default_handler` to any vector that:

- is not an exception with error code  
- is not the LAPIC timer vector  
- is not the syscall vector  
- does not already have a handler installed

### 2.6 Load the IDT

Finally:

```rust
unsafe { idt_ref().load(); }
```

This executes `lidt` and activates the interrupt subsystem.

---

## 3. Exception Handlers (Current Behavior)

All exception handlers:

- Log the exception name  
- Log the `InterruptStackFrame`  
- Log error codes where applicable  
- Panic the kernel  

### 3.1 Page Fault

`page_fault_handler` logs:

- Faulting address (`CR2`)
- `PageFaultErrorCode`
- Full stack frame

Then panics.

### 3.2 Double Fault

Runs on a dedicated IST stack.

Handler logs and panics.

### 3.3 General Protection Fault

Logs:

- Stack frame  
- Error code  

Then panics.

### 3.4 Invalid Opcode

Logs and panics.

### 3.5 Machine Check

Marked `-> !` (never returns).  
Logs and panics.

All other exceptions follow the same pattern.

---

## 4. LAPIC Timer Handler

Vector:

```
pub const LAPIC_TIMER_VECTOR: u8 = 0x31;
```

Handler:

```rust
extern "x86-interrupt" fn lapic_timer_handler(_stack_frame: InterruptStackFrame) {
    tick();
    send_eoi();
}
```

Behavior:

- Updates kernel tick counter  
- Sends EOI to LAPIC  
- Runs on IST for reliability  
- Does not log (to avoid spam)

---

## 5. Spurious Interrupt Handler

Vector:

```
0xFF
```

Handler:

```rust
extern "x86-interrupt" fn spurious_handler(_stack_frame: InterruptStackFrame) {
    error!("SPURIOUS INTERRUPT");
    send_eoi();
}
```

Logs and acknowledges the interrupt.

---

## 6. Default Handler

Any vector not explicitly assigned receives:

```rust
extern "x86-interrupt" fn default_handler(_stack_frame: InterruptStackFrame) {
    error!("UNHANDLED INTERRUPT");
}
```

This ensures no vector is left uninitialized.

---

## 7. Syscall Vector Integration

The syscall subsystem defines:

```
SYSCALL_VECTOR
```

`init_idt()` explicitly **skips** installing a default handler for this vector:

```rust
i == SYSCALL_VECTOR as usize
```

This allows the syscall subsystem to install its own handler.

Syscalls are not implemented via `int 0x80`.

---

## 8. Logging Behavior

The interrupt subsystem logs:

- Exception names  
- Stack frames  
- Error codes  
- Faulting addresses  
- Spurious interrupts  
- Unhandled vectors  
- Selected IDT entries during initialization  

This logging is essential for early kernel debugging.

---

## 9. Contributor Notes

- `init_idt()` must run before enabling interrupts  
- IST indices must match TSS configuration  
- LAPIC timer handler must call `send_eoi()`  
- Syscall vector must not be overwritten here  
- Keep handlers short and deterministic  
- Document new vectors when added  
- Avoid logging inside high‑frequency interrupts  

---

## License

MIT or Apache 2.0 — TBD.
