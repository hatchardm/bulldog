# 📖 Bulldog Kernel – Syscall Development Guide

This document replaces the former `apic.md` milestone guide.  
It provides technical context for contributors working on **privilege switching** and **syscall infrastructure** in the `feature/syscall` branch.

---

## 🧩 Purpose

The `feature/syscall` branch builds on the APIC baseline and introduces:

- Ring 0 ↔ Ring 3 privilege switching
- Syscall table and dispatcher
- Example syscalls for user ↔ kernel transitions
- Contributor visibility through logging and test harnesses

This guide explains the design decisions, implementation details, and contributor expectations.

---

## 🛠 Privilege Switching

### Goals
- Enable execution of user-mode code (Ring 3) while maintaining kernel-mode (Ring 0) isolation.
- Provide safe transitions between privilege levels using interrupts, exceptions, and syscalls.

### Implementation Steps
1. **GDT/TSS Setup**
   - Define separate segments for Ring 0 and Ring 3.
   - Configure Task State Segment (TSS) with kernel stack pointers.
2. **Stack Switching**
   - On privilege transitions, CPU loads the kernel stack from TSS.
   - Validate stack alignment for interrupt handling.
3. **Interrupt Handling**
   - Ensure IDT entries for syscalls and exceptions are configured with correct privilege levels.
   - Mask/unmask vectors as needed.

---

## 🔧 Syscall Infrastructure

### Dispatcher
- A central syscall handler receives requests from user mode.
- Syscall number indexes into a syscall table.

### Syscall Table
- Array of function pointers or match statement in Rust.
- Example entries:
  - `0x01` → framebuffer write
  - `0x02` → process yield
  - `0x03` → get system time

### Example Stub
pub fn syscall_dispatch(num: u64, arg1: u64, arg2: u64) -> u64 {
    match num {
        0x01 => framebuffer_write(arg1 as *const u8, arg2 as usize),
        0x02 => process_yield(),
        0x03 => system_time(),
        _    => error_unknown_syscall(num),
    }
}

---

## 🧪 Contributor Tasks

- Implement privilege switching logic in `arch/x86_64/syscall.rs`.
- Add at least one working syscall (e.g. framebuffer write).
- Extend logging to show syscall invocations.
- Document unsafe blocks with justification.

---

## 🧭 Roadmap

- [x] Paging and memory management  
- [x] Interrupt handling and IST setup  
- [x] GDT/TSS initialization  
- [x] APIC interrupt controller integration  
- [ ] Privilege switching (Ring 0 ↔ Ring 3)  
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

- Always test under QEMU before submitting PRs.
- Keep commits atomic and descriptive.
- Document new syscalls in this file (`docs/syscall.md`).
- Align contributions with the roadmap milestones.

---

## 📜 License

MIT or Apache 2.0 — TBD. Contributions welcome under either license.

