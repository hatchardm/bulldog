# Bulldog Kernel – Privilege Switching Guide


Ring 0 ↔ Ring 3 transitions for the feature/syscall branch.

This document provides technical context for contributors working on privilege switching in Bulldog. It complements docs/syscall.md by focusing on the mechanics of entering and exiting user mode.

## Prerequisites

Before implementing privilege switching, ensure:

- Paging is active and the higher-half kernel mapping is stable.

- GDT initialization is complete and selectors are not changing.

- TSS is allocated, initialized, and loaded via ltr.

- IDT is fully built and interrupts are enabled.

- Kernel stacks are allocated per-CPU (or per-core) and mapped.

## Purpose

Privilege switching allows Bulldog to:

- Execute user-mode code safely in Ring 3
- Maintain kernel isolation in Ring 0
- Provide a foundation for syscalls, processes, and scheduling

## Implementation Steps

### GDT Setup

- Kernel segments: Ring 0 code/data with DPL = 0

- User segments: Ring 3 code/data with DPL = 3

- Selectors: Allocate stable selectors for both kernel and user segments

- Keep selectors in a shared header to avoid drift

### TSS Configuration

- TSS rsp0 must point to the kernel stack used on Ring 3 → Ring 0 transitions

- Reserve at least one IST slot for critical faults

- On SMP, each core must have its own TSS and kernel stack

### IDT Entries

- Syscall gate uses an interrupt gate with DPL = 3

- Primary syscall vector is 0x80

- Exceptions remain DPL = 0

- Reserve vectors 0x80–0x8F for Bulldog syscalls

### Stack Switching

- CPU loads kernel stack from TSS on privilege transitions

- Maintain 16-byte alignment

- Place an unmapped guard page below each kernel stack

### Context Save and Restore

- Save user registers, flags, and user stack pointer

- Restore them on exit

- Ensure no kernel selectors or privileged flags leak back to user mode

- Save/restore SIMD/FPU state if needed

## Register Preservation Expectations

- General purpose: RAX, RBX, RCX, RDX, RSI, RDI, RBP, R8–R15
- Instruction/stack: RIP, RSP (user), return frame
- Flags: RFLAGS
- Segments: CS, SS (user)
- SIMD/FPU: XMM0–XMM15 (optional)


## Syscall Vector Allocation

- Primary syscall vector: 0x80

- Reserved range: 0x80–0x8F

- INT is used instead of SYSCALL/SYSRET because:

- SYSRET cannot return to non-canonical RIPs

- SYSRET has strict alignment and flag constraints

- iretq is simpler and safer for early development

## Return Path Mechanics

User Mode (Ring 3)
    |
    | int 0x80
    v
CPU pushes RIP, CS, RFLAGS, RSP, SS
CPU loads kernel RSP from TSS
    |
    v
Kernel Mode (Ring 0)
    |
    | iretq
    v
User Mode (Ring 3)


Entry: CPU builds a user frame and switches to the kernel stack  
Kernel: Saves context, handles the syscall, and prepares the return value  
Exit: iretq restores user‑mode state  
Invariant: No kernel state leaks back to user mode  


## Testing

A minimal user-mode program should verify:

- Correct Ring 3 → Ring 0 transition

- Kernel stack alignment

- Proper return via iretq

- Register preservation

- Logging of entry/exit breadcrumbs

## Example Outline

```rust
pub extern "C" fn privilege_switch_handler() {
    save_user_registers();
    kernel_task();
    restore_user_registers();
}
```


## Logging and Error Reporting Conventions

- Entry/exit tags: `PRIVSWITCH[entry]`, `PRIVSWITCH[exit]`

- Snapshot blocks: log  
  - CS  
  - SS  
  - RSP (user)  
  - RSP (kernel)  
  - RFLAGS  
  - RAX  

- Failure codes:
  - BAD_SELECTOR
  - STACK_MISALIGNED
  - GUARD_PAGE_FAULT
  - CONTEXT_LEAK


## Common Pitfalls

- Forgetting DPL = 3 on the syscall gate

- Misaligned kernel stack

- Returning to user mode with kernel CS/SS

- Not clearing AC or DF bits in RFLAGS

- Using the wrong TSS on SMP

- Forgetting guard pages

## Contributor Tasks

- Implement user-mode GDT entries

- Configure TSS and IST

- Add syscall gate(s)

- Build a minimal user-mode test program

## Roadmap Context

- [x] Paging
- [x] Interrupt handling
- [x] GDT/TSS
- [x] APIC
- [ ] Privilege switching
- [ ] Syscall interface
- [ ] Scheduling
- [ ] User mode execution

## License

- MIT or Apache 2.0 — TBD.
