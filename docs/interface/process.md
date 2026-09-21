# Bulldog Kernel – Process Interface

This document describes the user‑visible process model exposed by Bulldog’s interface layer.
It defines how processes are created, executed, terminated, and interacted with via syscalls.

It complements:

- `scheduler.md`
- `execve.md`
- `fork.md`
- `syscall-table.md`

---

## 1. Overview

A Bulldog process consists of:

- An address space  
- A set of CPU registers  
- A file descriptor table  
- A user stack  
- A kernel stack (TSS.rsp0)  
- A PID  

Processes interact with the kernel exclusively through syscalls.

---

## 2. Process Lifecycle

### Creation (future)
Processes will be created via:

- `fork()`  
- `execve()`  
- `spawn()` (future convenience wrapper)

### Execution
Processes run in Ring 3 and enter the kernel via:

- `syscall`
- Exceptions
- Interrupts

### Termination
Processes terminate via:

- `exit(code)`
- Fatal exceptions
- Kernel kill (future)

---

## 3. File Descriptor Table

Each process has a table of file descriptors:

- `0` – stdin  
- `1` – stdout  
- `2` – stderr  
- `>=3` – VFS handles  

FDs are inherited across `fork()` and replaced across `execve()`.

---

## 4. Address Space

Each process has:

- Code segment  
- Data segment  
- Heap  
- Stack  
- VDSO (future)  

Address spaces are created by:

- ELF loader  
- Page table builder  
- VFS subsystem (for mmap, future)

---

## 5. Syscalls

Processes interact with the kernel via:

- `read`  
- `write`  
- `open`  
- `close`  
- `alloc`  
- `free`  
- `exit`  
- `execve`  
- `fork` (future)

---

## 6. Future Enhancements

- Threads  
- Signals  
- Process groups  
- Shared memory  
- Copy‑on‑write  
- Per‑CPU scheduling  

---

## License
MIT or Apache 2.0 — TBD.
