# Bulldog Kernel – Early CPU Initialization

This document describes the CPU state before `kernel_init` runs, including register state,
paging assumptions, stack setup, and architectural invariants established by the bootloader.

It complements:

- `boot/bootloader.md`
- `gdt.md`
- `interrupts.md`
- `kernel_init.md`

---

## 1. CPU Mode

The bootloader enters the kernel in:

- 64‑bit long mode  
- Paging enabled  
- Identity map active  
- Interrupts disabled  
- GDT installed  
- A valid stack  
- CR3 pointing to bootloader page tables  

---

## 2. Register State

Typical initial register state:

- `RIP` → kernel entry point  
- `RSP` → bootloader‑provided stack  
- `CR0` → paging + protected mode  
- `CR4` → long mode extensions  
- `EFER` → LME + LMA  
- Segment registers set to flat 64‑bit model

---

## 3. Early Kernel Responsibilities

Before `kernel_init` runs, the kernel must:

1. Zero `.bss`  
2. Install kernel page tables  
3. Switch to higher‑half execution  
4. Initialize logging  
5. Initialize physical memory allocator  
6. Initialize direct physical map (HHDM)  
7. Prepare kernel stacks  
8. Initialize GDT/TSS  
9. Initialize IDT  
10. Enable interrupts (later)

---

## 4. Temporary Identity Map

The bootloader provides identity mapping for:

- Kernel physical load region  
- Low memory  
- Boot structures  

The kernel removes this mapping after switching to higher‑half paging.

---

## 5. Transition to Higher‑Half

The kernel performs:

- Page table installation  
- CR3 update  
- Far jump to higher‑half address  

This ensures:

- Kernel addresses are stable  
- User space cannot access kernel memory  
- Direct physical map is available

---

## 6. Future Enhancements

- Per‑CPU initialization  
- APIC base mapping  
- IST setup for exceptions  
- Early SMP boot coordination  
- KASLR randomization

---

## Contributor Notes

- Document all early‑boot assumptions  
- Keep CPU initialization deterministic  
- Avoid mixing bootloader and kernel responsibilities  
- Update this document when adding new CPU‑level features

---

## License
MIT or Apache 2.0 — to be determined.
