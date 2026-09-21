# Bulldog Kernel – Kernel Stacks and IST Stacks (Current Implementation)

This document describes Bulldog’s *current* kernel stack architecture, including the main
kernel stack, interrupt stacks (IST), and how they integrate with the GDT/TSS subsystem.
It reflects the real behavior of:

- `stack.rs` (static stack allocations)
- `gdt.rs` (TSS + IST configuration)
- `interrupts.rs` (IST‑backed handlers)
- `apic.rs` (LAPIC timer using IST1)

This is not a future design document — it describes exactly how Bulldog works today.

---

## 1. Overview

Bulldog uses three types of stacks:

1. **Main kernel stack**  
2. **IST0 – Double‑fault stack**  
3. **IST1 – LAPIC timer + page‑fault stack**

All stacks are:

- Statically allocated  
- 128 KiB in size  
- 16‑byte aligned  
- Located in higher‑half kernel space  
- Mapped with `PRESENT | WRITABLE`  
- Without guard pages (yet)

These stacks are defined in `stack.rs` and wired into the TSS in `gdt.rs`.

---

## 2. Static Stack Definitions

`stack.rs` defines two static stack regions:

```rust
#[repr(align(16))]
pub struct Stack(pub [u8; STACK_SIZE]);

pub const STACK_SIZE: usize = 128 * 1024;

pub static STACK: Stack = Stack([0; STACK_SIZE]);        // IST0 – double fault
pub static LAPIC_STACK: Stack = Stack([0; STACK_SIZE]);  // IST1 – LAPIC timer + page fault
```

These are placed in higher‑half memory by the linker.

---

## 3. IST Stack Usage

Bulldog uses two IST entries:

| IST Index | Purpose | Backing Stack |
|-----------|---------|---------------|
| **0** | Double fault | `STACK` |
| **1** | LAPIC timer + page fault | `LAPIC_STACK` |

### IST0 – Double Fault

Used exclusively for double‑fault exceptions.

A dedicated stack is required because double faults often occur due to:

- Stack corruption  
- Page‑fault recursion  
- Invalid kernel stack pointer  

IST0 ensures the CPU has a clean stack to panic safely.

### IST1 – LAPIC Timer + Page Fault

Used for:

- LAPIC timer interrupts  
- Page faults  

This prevents:

- Timer interrupts from running on a corrupted kernel stack  
- Page faults from reusing the same stack that triggered the fault  

IST1 dramatically improves robustness.

---

## 4. TSS Integration

`gdt.rs` installs the IST stacks into the TSS:

```rust
tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX] = df_stack_end;
tss.interrupt_stack_table[LAPIC_IST_INDEX] = lapic_stack_end;
```

The TSS is then loaded via:

```rust
load_tss(selectors.tss_selector);
```

This must occur **before enabling interrupts**.

---

## 5. Stack Mapping and Frame Marking

During `kernel_init`, Bulldog:

1. Computes the virtual range of each IST stack  
2. Marks all frames backing the stack as “used” in the bitmap allocator  
3. Remaps each page with correct flags  
4. Flushes TLB entries  

This ensures:

- IST stacks are fully mapped  
- No stack pages are reused  
- LAPIC timer and page‑fault handlers have safe stacks  

This is one of Bulldog’s strongest reliability features.

---

## 6. Main Kernel Stack

The main kernel stack is provided by the bootloader and used for:

- Kernel entry  
- `kernel_main`  
- `kernel_init`  
- Normal kernel execution  

It is **not** an IST stack.

Future work will introduce:

- Per‑CPU kernel stacks  
- Guard pages  
- Stack overflow detection  

---

## 7. Future Improvements

Planned enhancements:

- Guard pages around all stacks  
- Per‑CPU kernel stacks (SMP)  
- Per‑CPU IST stacks  
- Dedicated syscall entry stack (IST2)  
- Stack canaries  
- Red‑zone support  
- Better stack layout documentation  

---

## 8. Contributor Notes

- IST stacks must remain aligned and unmoved  
- Do not change stack sizes without updating `gdt.rs` and `kernel_init.md`  
- LAPIC timer and page‑fault handlers depend on IST1  
- Double‑fault handler depends on IST0  
- Guard pages are not implemented yet — do not assume their presence  

---

## License

MIT or Apache 2.0 — TBD.

