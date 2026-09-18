# Bulldog Kernel – Privilege Switching (Ring 3 ↔ Ring 0)

This document describes Bulldog’s *current* privilege‑switching architecture.  
It reflects the real behavior of:

- The syscall entry vector  
- IDT gate permissions  
- GDT/TSS configuration  
- Kernel stack switching  
- The `iretq` return path  
- The syscall harness validation

This is not a future design document — it describes exactly how Bulldog transitions between
user mode (Ring 3) and kernel mode (Ring 0) today.

---

## 1. Overview

Privilege switching is the mechanism that allows user‑mode code to safely enter the kernel
and return again. Bulldog uses:

- A dedicated syscall interrupt vector  
- A Ring‑3‑callable IDT gate  
- TSS‑based stack switching  
- A controlled register‑save/restore path  
- A strict `iretq` return frame

This ensures:

- User mode cannot corrupt kernel state  
- Kernel mode cannot accidentally return to an invalid user context  
- Syscalls behave deterministically  
- The syscall harness can validate correctness

---

## 2. Entering the Kernel (User → Kernel)

User mode enters the kernel via the syscall vector:

```
SYSCALL_VECTOR
```

This vector is:

- Installed by the syscall subsystem  
- **Not** installed by `interrupts::init_idt()`  
- Configured with DPL=3 (user mode can call it)  
- Routed onto the kernel stack via TSS IST switching

### Entry Sequence

1. User executes `int SYSCALL_VECTOR`
2. CPU checks IDT gate permissions (DPL=3)
3. CPU switches to the kernel stack using TSS
4. CPU pushes a privilege‑transition frame
5. Kernel entry stub saves user registers
6. Kernel calls `syscall_entry()`
7. Dispatcher runs
8. Kernel restores registers
9. Kernel returns via `iretq`

This is the only legal path from Ring 3 → Ring 0.

---

## 3. IDT Gate Permissions

The syscall vector’s IDT entry is configured with:

- **Present**  
- **Interrupt gate**  
- **DPL = 3** (user mode allowed)  
- **IST = kernel stack**  

This ensures:

- User mode can invoke the syscall  
- The CPU switches to a safe kernel stack  
- Interrupts are masked during entry  
- The return path is deterministic

---

## 4. TSS Stack Switching

Bulldog uses the TSS to switch stacks during privilege transitions.

The TSS contains:

- Kernel RSP0 (main kernel stack)
- IST stacks (double fault, LAPIC timer, page fault)

For syscalls, the CPU uses:

```
TSS.rsp0
```

This is the **kernel entry stack**.

### Why TSS stack switching?

Because user mode’s stack:

- Cannot be trusted  
- May be unmapped  
- May be corrupted  
- Must not be used for kernel execution

The TSS guarantees a clean kernel stack for every syscall.

---

## 5. Kernel Entry Stub

The entry stub (assembly) performs:

1. Save user registers  
2. Save user RSP  
3. Save user RIP  
4. Save user RFLAGS  
5. Build a syscall context frame  
6. Call `syscall_entry()` in Rust

The stub is intentionally minimal and deterministic.

---

## 6. Syscall Dispatcher

The dispatcher:

- Validates syscall number  
- Extracts arguments from registers  
- Calls the correct handler  
- Places return value in `rax`  
- Does **not** modify other registers

See:  
`dispatcher.md`  
`syscall.md`  
`syscall-table.md`

---

## 7. Returning to User Mode (Kernel → User)

Returning to user mode uses:

```
iretq
```

The return frame contains:

- User RIP  
- User CS (DPL=3)  
- User RFLAGS  
- User RSP  
- User SS (DPL=3)

The kernel must not corrupt this frame.

### Return Sequence

1. Kernel restores user registers  
2. Kernel loads user RSP  
3. Kernel loads user RIP  
4. Kernel executes `iretq`  
5. CPU switches back to Ring 3  
6. User mode resumes execution

This path is validated by the syscall harness.

---

## 8. Register Preservation

Bulldog’s syscall ABI requires:

- Only `rax` may be modified (return value)
- All other registers must be preserved
- Flags must be preserved
- Segment selectors must be preserved

The syscall harness tests:

- Register preservation  
- Correct return value  
- Correct privilege level  
- Correct stack switching  
- Correct `iretq` behavior  

This is one of Bulldog’s strongest invariants.

---

## 9. Security Guarantees

Privilege switching enforces:

- User mode cannot jump into kernel code  
- User mode cannot choose kernel stack pointers  
- User mode cannot forge return frames  
- Kernel mode cannot accidentally return to Ring 0  
- Kernel mode cannot return to invalid user memory

These guarantees are validated by:

- IDT gate permissions  
- TSS stack switching  
- Controlled register save/restore  
- Strict `iretq` return path  
- Syscall harness tests

---

## 10. Future Enhancements

Planned improvements:

- Dedicated syscall IST stack (IST2)  
- Per‑process kernel stacks  
- Per‑CPU kernel stacks (SMP)  
- Fast‑path syscalls (no dispatcher)  
- User‑mode signal delivery  
- Extended ABI for large arguments  
- Memory‑safe user pointer validation  

These will be documented once implemented.

---

## 11. Contributor Notes

- Syscall vector must have DPL=3  
- Syscall vector must not be installed by `interrupts::init_idt()`  
- TSS.rsp0 must always point to a valid kernel stack  
- Do not modify the privilege‑transition frame  
- All new syscalls must follow the ABI  
- Update the syscall harness when adding new syscalls  

---

## License

MIT or Apache 2.0 — TBD.
