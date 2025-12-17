# 📖 Bulldog Kernel – Privilege Switching Guide

This document provides technical context for contributors working on **Ring 0 ↔ Ring 3 privilege transitions** in the `feature/syscall` branch.  
It complements `docs/syscall.md` by focusing specifically on privilege switching mechanics.

---

## 🧩 Purpose

Privilege switching enables Bulldog to safely execute user-mode code (Ring 3) while maintaining kernel-mode (Ring 0) isolation.  
This is a critical milestone before full syscall infrastructure and user-mode execution.

---

## 🛠 Implementation steps

### 1. GDT setup
- **Kernel segments:** Define Ring 0 code/data with **DPL = 0**.
- **User segments:** Define Ring 3 code/data with **DPL = 3**.
- **Selectors:** Allocate stable selectors for `CS`/`DS` pairs; document them in a shared header to avoid drift.

### 2. TSS configuration
- **Kernel stack:** Configure TSS with the kernel stack pointer used on Ring 3 → Ring 0 transitions.
- **IST entries:** Reserve at least one IST slot for fault handlers that should bypass the current kernel stack (e.g., double fault).
- **Per-CPU TSS:** If SMP, ensure each core has its own TSS and stack to prevent cross-core corruption.

### 3. IDT entries
- **Syscall entry:** Mark the syscall gate as callable from Ring 3 (set DPL = 3 for the chosen vector).
- **Exceptions:** Route faults/exceptions to Ring 0 handlers (DPL typically 0), optionally using IST for critical faults.
- **Vector allocation:** Reserve a small range for Bulldog syscalls to avoid collisions with hardware IRQs (see “Syscall vector allocation” below).

### 4. Stack switching
- **Automatic switch:** On privilege transitions, the CPU loads the kernel stack from the TSS.
- **Alignment:** Maintain 16-byte alignment for the kernel stack per System V ABI expectations.
- **Guard pages:** Place an unmapped guard page below each kernel stack to catch overflows early; log violations via a distinct fault path.

### 5. Context save/restore
- **Preserve state:** Save user registers and `RFLAGS` on entry; restore them on exit to ensure a clean return to Ring 3.
- **Segment safety:** Validate user `CS`/`SS` and avoid leaking kernel segments back to user mode.
- **Floating point:** If user code uses SIMD/FPU, lazily save/restore `XMM`/`YMM` via `fxsave`/`fxrstor` (or a deferred scheme) to avoid performance cliffs.

---

## 🧾 Register preservation expectations

Contributors should ensure these registers/flags are preserved across the privilege switch:

| Group | Items |
| --- | --- |
| General purpose | `RAX`, `RBX`, `RCX`, `RDX`, `RSI`, `RDI`, `RBP`, `R8–R15` |
| Instruction/stack | `RIP`, `RSP` (user), return frame established by the gate |
| Flags | `RFLAGS` (carry, interrupt, direction, overflow, etc.) |
| Segments | `CS`, `SS` (user), validate selectors on return |
| SIMD/FPU (optional) | `XMM0–XMM15` via `fxsave`/`fxrstor` if needed |

> Tip: Keep the kernel entry stub minimal—do the absolute minimum before switching to a well-defined Rust handler that saves/restores context in one place. This reduces duplicated logic and surprise state.

---

## 🔢 Syscall vector allocation

- **Reserved range:** Bulldog reserves vectors `0x80–0x8F` for syscall entries.  
  - **Primary syscall:** `0x80` is the canonical “syscall gate” (INT-based) from Ring 3.
  - **Aux/control:** `0x81–0x8F` may be used for future fast paths or specialized gates.
- **Collision avoidance:** Do not place hardware IRQs or exception entries in `0x80–0x8F`. Keep hardware IRQs in the APIC range and exceptions at their standard vectors.
- **Why INT now:** Bulldog’s initial privilege switching uses an **interrupt gate + `iretq`** return path for clarity and contributor accessibility. `SYSCALL/SYSRET` (MSR-based) may be introduced later behind a feature flag.

---

## 🔄 Return path mechanics

- **Entry:** User-mode invokes `int 0x80` → CPU builds a user frame and switches to the kernel stack from TSS.
- **Kernel work:** Kernel saves user context, executes the handler, and prepares a return value in `RAX`.
- **Exit:** Kernel restores user context and returns with `iretq`, restoring `RIP`, `CS`, `RFLAGS`, `RSP`, `SS` to user-mode.
- **Invariant:** No kernel selectors or privileged flags should leak back to Ring 3; verify `IF` and `AC` bits in `RFLAGS` as needed.

---

## 🧪 Testing

Create a minimal user-mode program that triggers a syscall and verify:

- **Correct transition:** Ring 3 → Ring 0 entry occurs on `int 0x80`.
- **Kernel stack:** Kernel stack pointer changes to the TSS-defined stack; alignment remains 16-byte aligned.
- **Return path:** `iretq` restores user-mode execution and returns the expected value in `RAX`.
- **Logging:** Kernel logs include entry/exit breadcrumbs, register snapshots, and error codes on failure.

---

## 🔧 Example outline

```rust
pub extern "C" fn privilege_switch_handler() {
    // Save user context
    save_user_registers();

    // Kernel work (already on kernel stack via TSS)
    kernel_task();

    // Restore user context
    restore_user_registers();
}
```

---

## 🧪 Testing harness (drop-in)

```rust
#[repr(C)]
pub struct UserContext {
    rax: u64, rbx: u64, rcx: u64, rdx: u64,
    rsi: u64, rdi: u64, rbp: u64,
    r8: u64, r9: u64, r10: u64, r11: u64,
    r12: u64, r13: u64, r14: u64, r15: u64,
    rflags: u64, rip: u64, rsp_user: u64,
    cs: u64, ss: u64,
}

#[no_mangle]
pub extern "C" fn privilege_switch_handler(ctx: &mut UserContext) {
    log::info!("PRIVSWITCH[entry]");
    log::debug!(
        "ctx: CS={:#x} SS={:#x} RIP={:#x} RSP(user)={:#x} RFLAGS={:#x}",
        ctx.cs, ctx.ss, ctx.rip, ctx.rsp_user, ctx.rflags
    );

    ctx.rax = 0xBULLD0G;

    let ksp = current_kernel_rsp();
    if (ksp & 0xF) != 0 {
        log::error!("STACK_MISALIGNED: RSP(kernel)={:#x}", ksp);
    }

    log::info!("PRIVSWITCH[exit]");
}

#[inline(always)]
fn current_kernel_rsp() -> u64 {
    let rsp: u64;
    unsafe { core::arch::asm!("mov {}, rsp", out(reg) rsp) }
    rsp
}

#[no_mangle]
pub extern "C" fn user_entry() -> u64 {
    let ret: u64;
    unsafe {
        core::arch::asm!("int 0x80", out("rax") ret);
    }
    if ret == 0xBULLD0G {
        user_log_ok("Privilege switch OK");
    } else {
        user_log_err("Privilege switch FAILED");
    }
    ret
}

fn user_log_ok(msg: &str) { let _ = msg; }
fn user_log_err(msg: &str) { let _ = msg; }

pub fn init_syscall_gate(idt: &mut Idt) {
    const SYSCALL_VEC: u8 = 0x80;
    idt[SYSCALL_VEC as usize].set_interrupt_gate(privilege_switch_entry as u64);
    idt[SYSCALL_VEC as usize].set_selector(KERNEL_CODE_SELECTOR);
    idt[SYSCALL_VEC as usize].set_present(true);
    idt[SYSCALL_VEC as usize].set_dpl(3);
}
```

---

## 🧱 Logging and error reporting conventions

- **Entry/exit tags:** Use `PRIVSWITCH[entry]` and `PRIVSWITCH[exit]`.
- **Snapshot blocks:** Log `CS`, `SS`, `RSP(user)`, `RSP(kernel)`, `RFLAGS`, and `RAX`.
- **Failure codes:**  
  - **BAD_SELECTOR** – invalid user `CS`/`SS`.  
  - **STACK_MISALIGNED** – kernel stack not aligned.  
  - **GUARD_PAGE_FAULT** – stack overflow/underflow.  
  - **CONTEXT_LEAK** – kernel state leaked to user.  
- **Unsafe blocks:** Must justify assumptions and invariants.

---

## 🧰 Contributor tasks

- **Ring 3 GDT entries:** Implement and document user code/data segments.
- **TSS configuration:** Provide valid kernel stack pointers and IST reservations.
- **IDT entries:** Add syscall gate(s) with DPL = 3 and wire to kernel handlers.
- **Test harness:** Write a minimal user-mode program to validate switching and logging.

---

## 🧭 Roadmap context

- [x] Paging and memory management  
- [x] Interrupt handling and IST setup  
- [x] GDT/TSS initialization  
- [x] APIC interrupt controller integration  
- [ ] Privilege switching (Ring 0 ↔ Ring 3) ← **this milestone**  
- [ ] Syscall interface and dispatcher  
- [ ] Process scheduling  
- [ ] User mode execution  

---

## 🌐 Branching context

- `main` → APIC baseline (stable kernel)  
- `feature/pic8259` → legacy PIC baseline  
- `feature/apic` → APIC milestone  
- `feature/syscall` → privilege switching + syscall development (this branch)  

---

## 📜 License

MIT or Apache 2.0 — TBD. Contributions welcome under either license.

---

## Disclaimer

Bulldog and its subsystems (including syscalls, APIC, PIC8259, paging, and related features)  
are experimental and provided "as is" without warranty of any kind. They are intended for  
research, learning, and contributor experimentation. Running Bulldog on real hardware may  
expose quirks or limitations. Use at your own risk. The maintainers and contributors are  
not liable for any damages or issues arising from its use. By contributing or running Bulldog,  
you agree to abide by the terms of the project license.

