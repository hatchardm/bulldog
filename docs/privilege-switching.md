# 📖 Bulldog Kernel – Privilege Switching Guide

This document provides technical context for contributors working on **Ring 0 ↔ Ring 3 privilege transitions** in the `feature/syscall` branch.  
It complements `docs/syscall.md` by focusing specifically on privilege switching mechanics.

---

## 🧩 Purpose

Privilege switching enables Bulldog to safely execute user-mode code (Ring 3) while maintaining kernel-mode (Ring 0) isolation.  
This is a critical milestone before full syscall infrastructure and user-mode execution.

---

## 🛠 Implementation Steps

### 1. GDT Setup
- Define segment descriptors for Ring 0 (kernel) and Ring 3 (user).
- Ensure correct privilege level bits (DPL = 0 for kernel, DPL = 3 for user).

### 2. TSS Configuration
- Configure Task State Segment (TSS) with kernel stack pointers.
- CPU uses TSS to load the kernel stack when switching from Ring 3 → Ring 0.

### 3. IDT Entries
- Interrupt Descriptor Table (IDT) entries must specify correct privilege levels.
- Syscall interrupt vectors should be callable from Ring 3.
- Exceptions and faults must route to Ring 0 handlers.

### 4. Stack Switching
- On privilege transitions, CPU automatically switches to the kernel stack defined in TSS.
- Validate stack alignment and guard pages to prevent corruption.

### 5. Testing
- Create a minimal user-mode program that triggers a syscall.
- Verify:
  - Correct transition from Ring 3 → Ring 0.
  - Kernel stack is loaded.
  - Return path restores user-mode execution.

---

## 🔧 Example Outline

pub extern "C" fn privilege_switch_handler() {
    // Save user context
    save_user_registers();

    // Switch to kernel stack (handled by CPU via TSS)
    // Perform privileged operation
    kernel_task();

    // Restore user context
    restore_user_registers();
}

---

## 🧪 Contributor Tasks

- Implement GDT entries for Ring 3 segments.
- Configure TSS with valid kernel stack pointers.
- Add IDT entries for syscall and privilege-level transitions.
- Write a minimal user-mode test harness to validate switching.

---

## 🧭 Roadmap Context

- [x] Paging and memory management  
- [x] Interrupt handling and IST setup  
- [x] GDT/TSS initialization  
- [x] APIC interrupt controller integration  
- [ ] Privilege switching (Ring 0 ↔ Ring 3) ← **this milestone**  
- [ ] Syscall interface and dispatcher  
- [ ] Process scheduling  
- [ ] User mode execution  

---

## 📂 Branching Context

- `main` → APIC baseline (stable kernel)  
- `feature/pic8259` → legacy PIC baseline  
- `feature/apic` → APIC milestone  
- `feature/syscall` → privilege switching + syscall development (this branch)  

---

## 🤝 Contributor Notes

- Document unsafe blocks with justification.
- Test privilege switching under QEMU before submitting PRs.
- Keep commits atomic and descriptive.
- Align contributions with roadmap milestones.

---

## 📜 License

MIT or Apache 2.0 — TBD. Contributions welcome under either license.


## Disclaimer

Bulldog and its subsystems (including syscalls, APIC, PIC8259, paging, and related features)  
are experimental and provided "as is" without warranty of any kind. They are intended for  
research, learning, and contributor experimentation. Running Bulldog on real hardware may  
expose quirks or limitations. Use at your own risk. The maintainers and contributors are  
not liable for any damages or issues arising from its use. By contributing or running Bulldog,  
you agree to abide by the terms of the project license.

