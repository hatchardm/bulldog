# Bulldog Kernel – Kernel and IST Stacks (Current Implementation)

> **Disclaimer:**  
> This document describes Bulldog’s *current* stack implementation as it exists in the code today.  
> It does **not** cover future plans such as guard pages, per‑CPU stacks, user‑mode stacks,  
> or scheduler‑managed kernel stacks.  
>  
> For related subsystems, see:  
> - `gdt.md` (TSS and IST configuration)  
> - `interrupts.md` (exception and LAPIC timer handlers)  
> - `kernel_init.md` (stack mapping and initialization)  
> - `apic.md` (LAPIC timer integration)

Bulldog currently uses two statically allocated, higher‑half kernel stacks for critical interrupt handling.  
These stacks are used exclusively through the Interrupt Stack Table (IST) mechanism in the TSS.

---

# 1. Overview

Bulldog defines two dedicated stacks:

1. **Double Fault Stack**  
   - Used only for the double‑fault exception  
   - Lives in its own linker section (`.stack`)  
   - Exposed via `no_mangle` for TSS setup  
   - 16‑byte aligned  

2. **LAPIC IST Stack**  
   - Used for LAPIC timer interrupts  
   - Used for page‑fault exceptions  
   - Separate from the double‑fault stack  
   - 16‑byte aligned  

These stacks ensure that critical exceptions and timer interrupts execute on known‑good memory, even if the main kernel stack is corrupted.

---

# 2. Stack Structures

## 2.1 AlignedStack (Double Fault Stack)

```rust
#[repr(align(16))]
pub struct AlignedStack(pub [u8; STACK_SIZE]);
```

Properties:

- 16‑byte alignment (required by the x86‑64 ABI)
- Backing storage: `[u8; STACK_SIZE]`
- Used exclusively for the double‑fault IST entry

The global instance:

```rust
#[unsafe(link_section = ".stack")]
#[unsafe(no_mangle)]
pub static mut STACK: AlignedStack = AlignedStack([0; STACK_SIZE]);
```

### Why a dedicated section?

Placing the stack in `.stack` ensures:

- The linker can place it at a stable, predictable virtual address  
- The TSS can reference it directly  
- It is isolated from other kernel data  

### Stack start address

```rust
pub fn get_stack_start() -> VirtAddr {
    unsafe { VirtAddr::from_ptr(core::ptr::addr_of!(STACK.0)) }
}
```

This returns the *base* of the stack region; the TSS uses the *end* of the region as the initial RSP.

---

# 3. LAPIC IST Stack

The LAPIC IST stack is used for:

- LAPIC timer interrupts  
- Page‑fault exceptions  

Both handlers are configured in the IDT to use `LAPIC_IST_INDEX`.

Definition:

```rust
#[repr(align(16))]
pub struct Stack(pub [u8; STACK_SIZE]);

pub static LAPIC_STACK: Stack = Stack([0; STACK_SIZE]);
```

Properties:

- 16‑byte aligned  
- Statically allocated  
- Not placed in a special linker section  
- Mapped and protected during `kernel_init`  

### Why a separate stack?

Separating the LAPIC/page‑fault stack from the double‑fault stack:

- Prevents timer interrupts from consuming the double‑fault stack  
- Ensures page‑fault handlers run on clean memory  
- Provides isolation between failure contexts  

---

# 4. Stack Size and Alignment

`STACK_SIZE` is defined in `gdt.rs` and currently set to:

- **128 KiB per stack**

This size is chosen to:

- Provide ample space for nested exceptions  
- Avoid early stack exhaustion during debugging  
- Keep memory usage predictable  

All stacks are:

- **16‑byte aligned** (required by System V ABI)  
- **Statically allocated**  
- **Zero‑initialized**  

---

# 5. Integration With GDT and TSS

The stacks are wired into the TSS in `gdt.rs`:

- `DOUBLE_FAULT_IST_INDEX` → `STACK`  
- `LAPIC_IST_INDEX` → `LAPIC_STACK`  

This ensures:

- Double faults always run on a pristine, isolated stack  
- LAPIC timer and page‑fault handlers run on a stable stack even if the main kernel stack is corrupted  

---

# 6. Integration With kernel_init

`kernel_init` performs additional setup:

- Maps the LAPIC IST stack pages  
- Marks their frames as used in the frame allocator  
- Remaps them with correct flags (`PRESENT | WRITABLE`)  
- Logs the virtual range for debugging  

This ensures the LAPIC IST stack is fully backed by physical memory before interrupts are enabled.

---

# 7. Future Work

Planned improvements include:

- Guard pages below each stack  
- Per‑CPU kernel stacks (for SMP)  
- Per‑CPU IST stacks  
- User‑mode stacks and privilege switching  
- Dynamic stack allocation for threads/processes  
- Stack canaries and overflow detection  

These will be documented once implemented.

---

# 8. Contributor Notes

- Do not modify stack sizes without updating `gdt.rs` and `kernel_init.md`  
- All IST stacks must remain 16‑byte aligned  
- Avoid placing additional data in `.stack`  
- When adding new IST entries, document them here and in `gdt.md`  
- Keep stack usage minimal inside interrupt handlers  

---

# License

MIT or Apache 2.0 — TBD.
