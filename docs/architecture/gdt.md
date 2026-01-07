# Bulldog Kernel – Global Descriptor Table (GDT) and TSS

This document describes Bulldog’s current implementation of the Global Descriptor Table (GDT), Task State Segment (TSS), and Interrupt Stack Table (IST). It reflects the real behavior of:

- `gdt.rs` (GDT, TSS, IST setup)
- `stack.rs` (static IST stack allocations)
- `kernel_init` (GDT/TSS initialization)

This is a description of the *current* implementation, not future user‑mode or privilege‑switching plans.

---

# 1. Overview

Bulldog uses a minimal GDT configuration suitable for a 64‑bit kernel:

- Kernel code segment  
- Kernel data segment  
- TSS descriptor (required for IST support)

The TSS defines two dedicated IST stacks:

- IST0 → Double‑fault handler  
- IST1 → LAPIC timer + page‑fault handlers  

These stacks are statically allocated in higher‑half kernel space.

User‑mode segments are **not implemented yet**.

---

# 2. GDT Structure (Current)

The GDT is created in `gdt.rs` using `lazy_static!`:

- Kernel code segment (`Descriptor::kernel_code_segment()`)
- Kernel data segment (`Descriptor::kernel_data_segment()`)
- TSS segment (`Descriptor::tss_segment(&TSS)`)

The GDT is loaded once during early kernel initialization.

### Segment Selectors

`gdt.rs` stores the selectors in a small struct:

```rust
struct Selectors {
    code_selector: SegmentSelector,
    data_selector: SegmentSelector,
    tss_selector: SegmentSelector,
}
```

These selectors are used to load CS/DS/ES/SS and the TSS.

---

# 3. Task State Segment (TSS)

The TSS is defined using `lazy_static!` and contains two IST entries.

### IST Layout (Current)

| IST Index | Purpose | Backing Stack |
|-----------|---------|---------------|
| 0 | Double fault | `STACK` |
| 1 | LAPIC timer + page fault | `LAPIC_STACK` |

### Stack Properties

Each IST stack:

- Is 128 KiB  
- Is statically allocated in higher‑half kernel space  
- Is 16‑byte aligned (ABI requirement)  
- Uses the *end* of the region as the stack pointer  

Example from `gdt.rs`:

```rust
let df_stack_start = VirtAddr::from_ptr(unsafe { core::ptr::addr_of!(STACK.0) });
let df_stack_end = df_stack_start + STACK_SIZE;
tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX] = df_stack_end;
```

Guard pages are **not implemented yet**, but planned.

---

# 4. IST Usage in the Kernel

### IST0 – Double Fault

A double fault indicates:

- Stack corruption  
- Page‑fault recursion  
- Interrupt handling failure  

Using a dedicated IST ensures the CPU has a clean stack to recover or panic safely.

### IST1 – LAPIC Timer + Page Fault

Bulldog routes:

- LAPIC timer interrupts  
- Page faults  

…onto IST1.

This prevents:

- Timer interrupts from running on potentially corrupted kernel stacks  
- Page faults from reusing the same stack that triggered the fault  

This design improves robustness during early development.

---

# 5. GDT Initialization Flow

`init()` in `gdt.rs` performs:

1. Load GDT  
2. Load segment registers:  
   - `CS` → kernel code  
   - `DS`, `ES`, `SS` → kernel data  
3. Load TSS via `load_tss()`  

This must occur **before** enabling interrupts or LAPIC timers.

---

# 6. Interaction With Other Subsystems

### Memory Management

- IST stacks live in higher‑half kernel space  
- No guard pages yet (planned in memory subsystem)  
- LAPIC MMIO mapping must be active before LAPIC interrupts use IST1  

### APIC

- LAPIC timer interrupts use IST1  
- Page faults also use IST1  
- Double faults use IST0  

### Syscalls (Future)

User‑mode memory and privilege switching will require:

- Ring‑3 code/data segments  
- A syscall entry stack (possibly another IST entry)  
- A revised TSS layout  

These are not implemented yet.

---

# 7. Future Work

Planned improvements:

- Add guard pages around IST stacks  
- Add user‑mode segments (ring 3)  
- Add syscall entry stack (IST2)  
- Document stack layout in memory diagrams  
- Integrate with user‑mode page tables  

These changes will occur after the syscall subsystem is complete.

---

# 8. Contributor Notes

- IST stacks must remain aligned and unmoved  
- GDT/TSS initialization must occur before enabling interrupts  
- LAPIC timer and page‑fault handlers depend on IST1  
- Double‑fault handler depends on IST0  
- Do not modify stack sizes without updating alignment checks  

---

# License

MIT or Apache 2.0 — TBD.