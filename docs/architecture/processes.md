# Bulldog Kernel – Process Model and Context Switching

This document describes Bulldog’s planned process model, including task structures, context
switching, scheduling foundations, and user‑mode execution. While full process management is
a future milestone, this document defines the architectural direction and invariants that
contributors should follow as the subsystem evolves.

Bulldog targets the `x86_64-bulldog` architecture and uses a higher‑half kernel design with
strict separation between kernel and user space.

---

## Overview

Bulldog’s process subsystem is designed around the following goals:

- Clear separation between kernel and user execution  
- Deterministic context switching  
- Minimal, well‑defined process structures  
- Safe transitions between privilege levels  
- Compatibility with the syscall interface  
- Contributor‑friendly debugging and logging  

This document outlines the architecture that future scheduling and multitasking features
will build upon.

---

## Process Model (Planned)

Bulldog will use a simple process model:

- Each process has its own virtual address space  
- Each process has a user stack and heap  
- Kernel stacks are per‑CPU, not per‑process  
- Processes interact with the kernel exclusively through syscalls  
- Scheduling is cooperative at first, then preemptive via LAPIC timer  

A process will be represented by a `Process` structure containing:

- Page‑table root (CR3)  
- User stack pointer  
- User instruction pointer  
- Process ID  
- State (running, waiting, terminated)  
- Kernel bookkeeping fields  

This structure is intentionally minimal to keep early development simple.

---

## Thread Model (Future)

Bulldog will eventually support threads:

- Each thread has its own user stack  
- Threads share a process address space  
- Kernel stacks remain per‑CPU  
- Scheduling decisions operate at the thread level  

Thread support will be added after basic process scheduling is stable.

---

## Context Switching

Context switching is the act of saving the current CPU state and restoring another.  
Bulldog will use a deterministic, minimal context frame.

A context switch must save:

- General‑purpose registers  
- Instruction pointer (RIP)  
- Stack pointer (RSP)  
- Segment selectors (CS, SS for user mode)  
- RFLAGS  

Context switching will occur:

- On syscall return (cooperative scheduling)  
- On LAPIC timer interrupt (preemptive scheduling, future)  
- When a process blocks or exits  

Example context frame:

```
struct Context {
    r15, r14, r13, r12,
    r11, r10, r9, r8,
    rsi, rdi, rbp, rdx,
    rcx, rbx, rax,
    rip, rsp, rflags,
    cs, ss
}
```

This frame is stored on the kernel stack during a switch.

---

## Scheduling

Bulldog’s scheduler will evolve in stages:

### Stage 1: Cooperative Scheduling

- Processes yield explicitly via a syscall  
- No timer interrupts  
- Simple round‑robin or single‑task execution  

### Stage 2: Preemptive Scheduling (Future)

- LAPIC timer interrupts trigger scheduling  
- Kernel saves current context  
- Next runnable process is selected  
- Context is restored and execution continues  

### Stage 3: Advanced Scheduling (Future)

- Priorities  
- Sleep queues  
- Blocking I/O  
- Multi‑CPU load balancing  

The scheduler will be modular to support incremental development.

---

## User‑Mode Execution

User‑mode execution requires:

- A valid user stack  
- A valid user instruction pointer  
- A process‑specific page table  
- Correct GDT/TSS configuration  
- Correct IDT entries for syscalls and interrupts  

Transition to user mode uses:

```
iretq
```

with a frame containing:

- User RIP  
- User CS (DPL=3)  
- RFLAGS  
- User RSP  
- User SS (DPL=3)  

This is the same mechanism used for syscall returns.

---

## Syscall Interaction

Syscalls are the only legal entry point from user mode into the kernel.

Syscalls:

- Save user context  
- Switch to kernel stack via TSS  
- Execute kernel logic  
- Optionally trigger a context switch  
- Return via `iretq`  

Syscalls must not:

- Modify user registers except return value  
- Corrupt user stack or memory  
- Leave interrupts disabled  

See:  
`../syscall.md`  
`../privilege-switching.md`  
`../syscall-harness-guide.md`

---

## Process Creation (Future)

Process creation will involve:

1. Allocating a new page table  
2. Mapping user code and data  
3. Allocating a user stack  
4. Initializing a context frame  
5. Adding the process to the scheduler  

Later enhancements:

- ELF loader  
- Copy‑on‑write fork  
- Exec system call  

---

## Process Termination

A process may terminate via:

- `sys_exit`  
- Fatal exception  
- Kernel‑initiated kill (future)  

Termination steps:

- Mark process as terminated  
- Free user memory regions  
- Free page tables  
- Remove from scheduler  

---

## Contributor Notes

- Keep process structures minimal and well‑documented  
- Ensure context switching remains deterministic  
- Avoid storing kernel pointers in user‑accessible memory  
- Document all new scheduling states and transitions  
- Maintain compatibility with the syscall interface  

---

## License

MIT or Apache 2.0 — to be determined.
