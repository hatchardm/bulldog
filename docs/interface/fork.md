# Bulldog Kernel – `fork` Syscall (Planned)

This document describes the planned `fork` syscall, which creates a new process by duplicating
the calling process. Bulldog will implement `fork` after the scheduler and user‑mode memory
model are complete.

It complements:

- `exec.md`
- `execve.md`
- `process.md`
- `scheduler.md`

---

## 1. Purpose

`fork` creates a new process with:

- A copy of the parent’s address space  
- A copy of the parent’s registers  
- A copy of the parent’s file descriptor table  
- A new PID  

Parent receives child PID.  
Child receives `0`.

---

## 2. Proposed ABI

```
fork() -> SysResult<u64>
```

Returns:

- Child PID (parent)  
- `0` (child)  
- Error code on failure  

---

## 3. Kernel Responsibilities

When `fork` is invoked:

1. Duplicate page tables  
2. Duplicate memory regions  
3. Duplicate FD table  
4. Duplicate process structure  
5. Add child to scheduler  
6. Return appropriate values  

Bulldog will use **copy‑on‑write** in a future milestone.

---

## 4. Interaction With Scheduler

Child process is placed in the ready queue.

Parent continues execution immediately.

---

## 5. Error Handling

Possible errors:

- Out of memory  
- Too many processes  
- Page table duplication failure  
- FD duplication failure  

---

## 6. Future Enhancements

- Copy‑on‑write  
- Shared memory regions  
- Thread creation (`clone`)  
- Process groups  
- Signals  

---

## License
MIT or Apache 2.0 — to be determined.
