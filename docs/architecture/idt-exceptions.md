# Bulldog Kernel – IDT Exceptions

This document describes Bulldog’s exception handling architecture: how exceptions are routed
through the IDT, how the kernel manages fault conditions, and how exception frames integrate
with privilege switching and future user‑mode execution.

It complements:

- `interrupts.md`
- `privilege-switching.md`
- `paging.md`
- `process.md`

---

## 1. Overview

Exceptions are synchronous faults raised by the CPU. They occur during instruction execution
and must be handled deterministically.

Bulldog installs exception handlers in the IDT during early initialization. Each handler:

- Receives a CPU‑generated exception frame  
- Logs diagnostic information  
- Determines whether the fault is recoverable  
- Either resolves the fault or terminates the current process (future)  

---

## 2. Exception Categories

### Faults  
Recoverable if corrected.  
Examples: page fault, general protection fault.

### Traps  
Reported immediately after the instruction.  
Examples: debug trap, breakpoint.

### Aborts  
Unrecoverable.  
Examples: double fault, machine check.

---

## 3. Exception Vector Layout

| Vector | Name                     | Notes |
|--------|---------------------------|-------|
| 0      | Divide Error              | Fault |
| 1      | Debug                     | Trap  |
| 2      | NMI                       | Interrupt |
| 3      | Breakpoint                | Trap |
| 4      | Overflow                  | Trap |
| 5      | Bound Range               | Fault |
| 6      | Invalid Opcode            | Fault |
| 7      | Device Not Available      | Fault |
| 8      | Double Fault              | Abort (IST required) |
| 9      | Coprocessor Segment       | Fault |
| 10     | Invalid TSS               | Fault |
| 11     | Segment Not Present       | Fault |
| 12     | Stack Fault               | Fault |
| 13     | General Protection Fault  | Fault |
| 14     | Page Fault                | Fault |
| 16     | x87 Floating Point        | Fault |
| 17     | Alignment Check           | Fault |
| 18     | Machine Check             | Abort |
| 19     | SIMD Floating Point       | Fault |

Bulldog installs handlers for all mandatory exceptions.

---

## 4. Exception Frames

The CPU pushes an exception frame containing:

- RIP  
- CS  
- RFLAGS  
- RSP  
- SS  
- Error code (for some exceptions)  

Bulldog’s handler receives this frame and may:

- Log it  
- Inspect faulting address (CR2 for page faults)  
- Terminate the current process (future)  
- Panic the kernel  

---

## 5. Page Fault Handling

Page faults provide:

- Faulting virtual address (CR2)  
- Error code bits:  
  - Present  
  - Write  
  - User  
  - Reserved bit  
  - Instruction fetch  

Future Bulldog versions will:

- Kill the offending process  
- Provide user‑mode diagnostics  
- Support demand paging  

---

## 6. Double Fault Handling

Double faults require an IST entry in the TSS.

Bulldog configures:

- A dedicated double‑fault stack  
- A dedicated IDT entry  
- A panic handler  

Double faults are unrecoverable.

---

## 7. Future Enhancements

- Per‑process exception reporting  
- User‑mode signal delivery  
- Page fault recovery  
- Stack overflow detection  
- Guard pages  
- Exception logging buffers  

---

## License
MIT or Apache 2.0 — to be determined.
