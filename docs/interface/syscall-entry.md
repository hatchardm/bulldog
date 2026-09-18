# Bulldog Kernel – Syscall Entry Path

This document describes the syscall entry mechanism used by Bulldog, including the syscall
vector, MSR configuration, privilege switching, and the transition from user mode to kernel
mode.

It complements:

- `syscall.md`
- `syscall-table.md`
- `dispatcher.md`
- `privilege-switching.md`
- `exec.md`
- `execve.md`

---

## 1. Overview

Syscalls provide controlled entry into the kernel from user mode. Bulldog uses the x86_64
`syscall` / `sysret` instruction pair for fast privilege switching.

Syscall entry is routed through a dedicated IDT vector.

---

## 2. Syscall Vector

Bulldog reserves a vector for syscall entry:

```
SYSCALL_VECTOR
```

This vector is excluded from default handlers in `interrupts.rs`.

Syscalls do not use the IDT directly; they use MSRs to define entry points.

---

## 3. Required MSRs

Bulldog configures:

### `STAR`
Defines segment selectors for syscall entry and exit.

### `LSTAR`
Defines the 64‑bit syscall entry point.

### `FMASK`
Defines which RFLAGS bits are cleared on syscall entry.

These MSRs are configured during syscall initialization.

---

## 4. Entry Path

When user mode executes:

```
syscall
```

The CPU performs:

1. Save RIP → RCX  
2. Save RFLAGS → R11  
3. Load kernel CS/SS from `STAR`  
4. Jump to `LSTAR` entry point  
5. Switch to kernel stack (via TSS)  
6. Begin executing syscall handler  

Bulldog’s syscall entry point:

- Saves volatile registers  
- Builds a syscall frame  
- Dispatches to the syscall table  
- Restores registers  
- Returns via `sysret`  

---

## 5. Syscall Frame

The syscall frame contains:

- User RIP  
- User RFLAGS  
- User RCX  
- User R11  
- Arguments in registers  
- Kernel stack pointer  

This frame is used by the dispatcher.

---

## 6. Dispatcher

The dispatcher:

1. Reads syscall number  
2. Validates it  
3. Calls the corresponding handler  
4. Returns result in RAX  

Syscall table is documented in `syscall-table.md`.

---

## 7. Return Path

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

- User RIP to be canonical  
- User CS to be a valid user segment  
- RFLAGS to be sanitized  

---

## 8. Future Enhancements

- Per‑thread syscall stacks  
- Syscall filtering  
- Syscall tracing  
- Syscall restart  
- VDSO support  
- Fast user‑mode time queries  

---

## License
MIT or Apache 2.0 — to be determined.
