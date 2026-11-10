## Bulldog Kernel – exit Syscall

Reviewed: November 2026

This document describes Bulldog's current process-exit syscall.

The exit syscall records process termination state and performs resource cleanup.

Related files:

    kernel/src/syscall/exit.rs
    process.md
    syscall.md
    syscall-table.md

---

## 1. Purpose

The exit syscall terminates the current user process.

Current syscall number:

    2

The syscall records the process exit code and performs cleanup of process-owned resources.

---

## 2. Prototype

```text
exit(code: u64) -> int
```

Arguments:

    code
        Process exit status

Return value:

    0 on success

The current implementation returns success after cleanup.

---

## 3. Implementation

Current implementation:

```rust
sys_exit(code)
```

Operation flow:

    Record exit code
            ↓
    Mark process exited
            ↓
    Clear file descriptors
            ↓
    Perform configured post-exit behavior
            ↓
    Return success

---

## 4. Process State Updates

The syscall records:

    PID
    Exit code

using:

```rust
mark_exited(pid, code)
```

This marks the process as exited within the process subsystem.

---

## 5. File Descriptor Cleanup

Current exit processing clears process file descriptors.

Depending on execution mode:

```text
fd_clear_all()
```

or

```text
fd_reset_for_harness()
```

is performed.

The syscall harness validates that the file-descriptor table is empty following exit.

---

## 6. Exit Modes

### Normal Harness Mode

Current default behavior:

```text
Mark process exited
Clear FD table
Return success
```

This allows syscall validation to continue.

---

### Idle Mode

When armed for idle transition:

```text
Mark process exited
Clear FD table
Unmap user memory
Enter kernel idle state
```

---

### Goodbye Chaining Mode

When compiled with:

```text
usermode_chain
```

and armed appropriately:

```text
Mark process exited
Clear FD table
Unmap user memory
Run /bin/goodbye
```

This supports user-mode program chaining.

---

## 7. Harness Coverage

The syscall harness validates:

### Normal Exit

```text
exit(123)
```

Expected:

```text
Process marked exited
FD table cleared
Return 0
```

### Maximum Exit Code

```text
exit(u64::MAX)
```

Expected:

```text
Successful exit handling
```

### FD Cleanup

The harness verifies:

```text
FD table empty after exit
```

---

## 8. Current Limitations

Current implementation:

    Does not terminate the kernel
    Does not destroy an address space
    Does not perform scheduler removal here
    Uses shared kernel infrastructure

These behaviors may evolve as process management matures.

---

## 9. Future Enhancements

Planned improvements:

    Per-process address spaces
    Scheduler integration
    Parent/child exit tracking
    Wait semantics
    Exit notifications
    Resource accounting

---

## Contributor Notes

- Preserve exit-code recording.
- Preserve FD cleanup behavior.
- Keep harness support intact.
- Review this document when process-lifecycle behavior changes.
- Synchronize updates with process documentation.

---

## License

MIT or Apache 2.0 — TBD.
