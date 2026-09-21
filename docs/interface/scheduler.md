# Bulldog Kernel – Scheduler Interface

This document describes the scheduler interface exposed to user mode and kernel subsystems.
It complements the architecture‑level scheduler design (future) and defines how processes
interact with scheduling.

It complements:

- `process.md`
- `exec.md`
- `fork.md`
- `syscall.md`

---

## 1. Overview

The scheduler determines which process runs on the CPU. Bulldog’s scheduler interface defines:

- How processes yield  
- How processes block  
- How processes are selected  
- How scheduling interacts with syscalls  

The architecture‑level scheduler (future) will implement the actual context switching.

---

## 2. Scheduling Model

### Stage 1 — Cooperative (current)
- Processes yield voluntarily  
- No timer interrupts  
- Round‑robin selection  

### Stage 2 — Preemptive (future)
- LAPIC timer interrupts  
- Kernel saves context  
- Next runnable process selected  

### Stage 3 — Advanced (future)
- Priorities  
- Sleep queues  
- Blocking I/O  
- Multi‑CPU load balancing  

---

## 3. Scheduler Interface (Kernel)

Kernel subsystems may:

- Add a process to run queue  
- Remove a process  
- Mark a process as waiting  
- Mark a process as runnable  
- Trigger a reschedule  

These functions are internal to the kernel.

---

## 4. Scheduler Interface (User Mode)

User mode interacts with scheduling via syscalls:

### `yield` (future)
Voluntarily yield CPU.

### `sleep` (future)
Block for a duration.

### `wait` (future)
Block until event or child process exit.

### `exec` / `execve`
Replace current process image.

### `exit`
Terminate process.

---

## 5. Process States

- `Ready`  
- `Running`  
- `Waiting`  
- `Terminated`  

The scheduler transitions processes between these states.

---

## 6. Future Enhancements

- Per‑CPU run queues  
- Priority scheduling  
- Fair scheduling  
- Blocking I/O integration  
- Signals  
- Thread support  

---

## License
MIT or Apache 2.0 — to be determined.
