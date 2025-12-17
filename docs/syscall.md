# 📖 Bulldog Kernel – Syscall Development Guide

**Version:** v0.1-pre  
**Updated:** 2025-11-30  

This document replaces the former `apic.md` milestone guide.  
It provides technical context for contributors working on privilege switching and syscall infrastructure in the `feature/syscall` branch.

---

## 🏛 Milestone Lineage
- `feature/pic8259` → Legacy PIC baseline  
- `feature/apic` → Modern APIC baseline  
- `feature/syscall` → Privilege switching + syscall interface  

---

## 🧩 Purpose
The `feature/syscall` branch builds on the APIC baseline and introduces:
- Ring 0 ↔ Ring 3 privilege switching  
- Syscall table and dispatcher  
- Example syscalls for user ↔ kernel transitions  
- Contributor visibility through logging and test harnesses  

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

## 🌐 Syscall Model Overview

Bulldog’s syscall design is secure, scalable, and contributor‑friendly.  
Applications never call raw interrupts or numeric IDs directly. Instead, they use a **wrapper library** that exposes clean, named functions (e.g. `open_file()`, `spawn_process()`).

### Lifecycle
1. **Wrapper Library (User Space)**  
   - Validates arguments, attaches capability token, logs metadata.  
2. **Dispatcher (Kernel Boundary)**  
   - Receives request, verifies token, checks syscall table integrity, queues request.  
3. **Worker Pool (Kernel Space)**  
   - Executes handler under strict security policies, returns result.  
4. **Response Path**  
   - Dispatcher sends result back, wrapper logs completion, application continues.  

### Benefits
- **Security**: Capability tokens prevent arbitrary syscall abuse.  
- **Auditability**: Logging at both wrapper and dispatcher levels creates a clear trail.  
- **Scalability**: Worker pool avoids one‑to‑one thread pairing overhead.  
- **Contributor Hygiene**: Developers see a simple API, while the kernel enforces strict contracts invisibly.  

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
```rust
pub fn syscall_dispatch(num: u64, arg1: u64, arg2: u64) -> u64 {
    match num {
        0x01 => framebuffer_write(arg1 as *const u8, arg2 as usize),
        0x02 => process_yield(),
        0x03 => system_time(),
        _    => error_unknown_syscall(num),
    }
}
```

---

## 📞 Calling Convention
- **rax** → syscall number  
- **rdi, rsi, rdx, r10, r8, r9** → arguments (up to 6)  
- **rax** → return value  
- Preserve `rbp`, `rbx`, `r12–r15` (callee-saved)  
- Errors returned via `rax` using negative codes  

---

## 🧪 Contributor Tasks & Hygiene
- Implement privilege switching logic in `arch/x86_64/syscall.rs`.  
- Add at least one working syscall (e.g. framebuffer write).  
- Extend logging to show syscall invocations.  
- Document unsafe blocks with justification.  
- Update `docs/syscall.md` whenever a new syscall is added.  
- Test under QEMU before submitting PRs.  
- Keep commits atomic and descriptive.  
- Justify all `unsafe` blocks with comments explaining invariants.  
- Align contributions with roadmap milestones.  
- Maintain branch hygiene (tags, clear forks from APIC baseline).  

---

## 📊 Syscall Number Allocation Table

| Number | Name             | Status        | Notes                               |
|--------|------------------|---------------|-------------------------------------|
| 1      | sys_write        | Implemented   | Writes buffer to fd                 |
| 2      | sys_exit         | Implemented   | Terminates process                  |
| 3      | sys_open         | Implemented   | Opens file descriptor               |
| 4      | sys_read         | Stub example  | Reads buffer (to be implemented)    |
| 5–15   | Reserved         | Future use    | Suggested for core POSIX calls      |
| 16–31  | Reserved         | Future use    | Extended Bulldog syscalls           |
| 32+    | Experimental     | Contributor proposals | Document in `docs/syscall.md` |

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

## ✅ Milestone Success Criteria
- Syscall handler registered at vector 0x80  
- Entry trigger logs dispatch correctly  
- Returns cleanly to caller  
- At least one syscall implemented and documented  

---

## 📜 License
MIT or Apache 2.0 — TBD. Final license choice will be documented before v0.1 release.  
Contributions welcome under either license.  

---

## Disclaimer
Bulldog and its subsystems (including syscalls, APIC, PIC8259, paging, and related features)  
are experimental and provided "as is" without warranty of any kind. They are intended for  
research, learning, and contributor experimentation. Running Bulldog on real hardware may  
expose quirks or limitations. Use at your own risk. The maintainers and contributors are  
not liable for any damages or issues arising from its use. By contributing or running Bulldog,  
you agree to abide by the terms of the project license.

