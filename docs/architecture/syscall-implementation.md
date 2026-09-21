# Bulldog Kernel – Syscall Implementation Details

This document describes the implementation details of Bulldog’s syscall subsystem, including
MSR configuration, entry/exit path, register handling, and dispatcher behavior.

It complements:

- `syscall-entry.md` (architecture)
- `syscall.md`
- `syscall-table.md`
- `dispatcher.md`
- `privilege-switching.md`

---

## 1. Overview

Bulldog uses the x86_64 `syscall` / `sysret` instruction pair for fast user‑mode → kernel‑mode
transitions.

Syscalls do not use the IDT.  
They use MSRs to define:

- Entry point
- Exit point
- Segment selectors
- RFLAGS mask

---

## 2. Required MSRs

Bulldog configures:

### `STAR`
Defines:

- Kernel CS/SS for syscall entry
- User CS/SS for syscall exit

### `LSTAR`
Defines the 64‑bit syscall entry point.

### `FMASK`
Defines which RFLAGS bits are cleared on entry.

Bulldog clears:

- Interrupt flag (IF)
- Trap flag (TF)

---

## 3. Entry Path

When user mode executes `syscall`, the CPU:

1. Saves RIP → RCX  
2. Saves RFLAGS → R11  
3. Loads kernel CS/SS from `STAR`  
4. Loads kernel RIP from `LSTAR`  
5. Switches to kernel stack (TSS.rsp0)  
6. Begins executing syscall entry code  

Bulldog’s entry code:

- Saves volatile registers
- Builds a syscall frame
- Dispatches based on syscall number
- Restores registers
- Returns via `sysretq`

---

## 4. Syscall Frame

The syscall frame contains:

- User RIP  
- User RFLAGS  
- User RCX  
- User R11  
- Arguments in registers  
- Kernel stack pointer  

This frame is passed to the dispatcher.

---

## 5. Dispatcher

The dispatcher:

1. Reads syscall number from RAX  
2. Validates it  
3. Calls the corresponding handler  
4. Returns result in RAX  

Errors are returned as negative values.

---

## 6. Return Path

Syscalls return via:

```
sysretq
```

This restores:

- User RIP  
- User RFLAGS  
- User RCX  
- User R11  
- User stack  

Sysret requires:

- Canonical user RIP  
- Valid user CS  
- Sanitized RFLAGS  

---

## 7. Future Enhancements

- Dedicated syscall IST stack
- Syscall filtering
- Syscall tracing
- Restartable syscalls
- VDSO support
- Fast user‑mode time queries

---

## License
MIT or Apache 2.0 — TBD.
