## Bulldog Kernel – Syscall Dispatcher (Current Implementation)

Reviewed: November 2026

This document describes Bulldog's current syscall dispatcher architecture.

It reflects the implementation found in:

    kernel/src/syscall/dispatcher.rs
    kernel/src/syscall/table.rs
    kernel/src/tests/syscall_harness.rs

It complements:

    syscall.md
    syscall-table.md
    syscall-entry.md
    privilege-switching.md

This document describes the current implementation rather than future designs.

---

## 1. Overview

The syscall dispatcher is responsible for:

    Receiving control from user mode
    Extracting syscall arguments
    Looking up handlers
    Invoking the correct syscall
    Returning results to user mode

The dispatcher is intentionally small and deterministic.

Current syscall entry vector:

    0x80

Unknown syscalls return:

    ENOSYS

rather than generating a fault.

---

## 2. Architecture

The dispatcher consists of three layers:

    syscall_handler()
            ↓
    rust_dispatch()
            ↓
    dispatch()

Responsibilities:

    syscall_handler()
        Low-level interrupt entry

    rust_dispatch()
        Rust-facing bridge and logging

    dispatch()
        Table lookup and handler invocation

---

## 3. Syscall Entry Path

User-mode code invokes:

    int 0x80

The CPU then performs:

    Ring 3
            ↓
    IDT gate validation
            ↓
    Ring 0 transition
            ↓
    syscall_handler()
            ↓
    rust_dispatch()
            ↓
    dispatch()
            ↓
    Syscall implementation

This behavior is configured using:

    SYSCALL_VECTOR = 0x80

The syscall gate is installed with:

    DPL = 3

allowing Ring 3 code to invoke syscalls.

---

## 4. Low-Level Entry Stub

Syscall entry begins in:

```rust
syscall_handler()
```

implemented as a naked function.

Responsibilities:

    Preserve registers
    Rearrange syscall arguments
    Call rust_dispatch()
    Return via iretq

The handler preserves:

```text
rbx
rbp
r12
r13
```

before transferring control to Rust code.

The entry stub should remain:

    Minimal
    Deterministic
    Easy to audit

---

## 5. Argument Mapping

The current syscall ABI uses:

<table>
<tr>
<th>Register</th>
<th>Purpose</th>
</tr>

<tr>
<td>rax</td>
<td>Syscall number</td>
</tr>

<tr>
<td>rdi</td>
<td>arg0</td>
</tr>

<tr>
<td>rsi</td>
<td>arg1</td>
</tr>

<tr>
<td>rdx</td>
<td>arg2</td>
</tr>

</table>

The dispatcher ultimately invokes handlers using:

```rust
fn(u64, u64, u64) -> u64
```

Arguments are forwarded as:

```text
arg0
arg1
arg2
```

---

## 6. Rust Dispatch Layer

The Rust entry bridge is:

```rust
rust_dispatch()
```

Responsibilities:

    Optional syscall-test logging
    Dispatch invocation
    Return-value propagation

When the syscall_tests feature is enabled, the dispatcher logs:

```text
Syscall number
Arguments
Return value
```

This output is used by the syscall harness.

---

## 7. Dispatcher Flow

Dispatcher implementation:

```rust
pub fn dispatch(
    num: u64,
    arg0: u64,
    arg1: u64,
    arg2: u64,
) -> u64
```

Dispatch process:

    Syscall number
            ↓
    lookup(num)
            ↓
      Found?
      /    \
    Yes     No
     |       |
     ↓       ↓
 Handler   ENOSYS
     |
     ↓
 Return

The dispatcher contains no syscall-specific logic.

All syscall behavior resides in individual syscall handlers.

---

## 8. Table Lookup

Handler lookup uses:

```rust
lookup(num)
```

provided by:

```text
kernel/src/syscall/table.rs
```

The syscall table is:

    Static
    Compile-time initialized
    Read-only

Current implemented syscalls:

```text
1  write
2  exit
3  open
4  read
5  alloc
6  free
7  close
```

All other table entries are currently unimplemented.

---

## 9. Error Handling

Unknown syscall numbers return:

```text
ENOSYS
```

Example:

```text
999
```

Result:

```text
ENOSYS
```

The syscall harness validates this behavior.

Other errors are produced by individual syscall handlers:

```text
EBADF
EFAULT
EINVAL
EMFILE
ENOMEM
```

The dispatcher itself only generates:

```text
ENOSYS
```

for unknown syscall numbers.

---

## 10. Return Path

After syscall execution:

    Handler returns
            ↓
    dispatch() returns
            ↓
    rust_dispatch() returns
            ↓
    syscall_handler()
            ↓
    iretq
            ↓
    Ring 3 resumes

Return values are placed in:

```text
rax
```

Execution resumes after the original:

```text
int 0x80
```

instruction.

---

## 11. Harness Coverage

The syscall harness validates:

### Dispatcher operation

    Correct syscall lookup
    Correct handler invocation
    Correct return values

### Error paths

    Unknown syscall → ENOSYS

### Syscall table coverage

    write
    exit
    open
    read
    alloc
    free
    close

### Privilege switching

    Ring 3 entry
    Ring 0 execution
    iretq return

The syscall harness serves as the primary behavioral validation source for the dispatcher.

---

## 12. Current Limitations

Current implementation:

    Supports three syscall arguments
    Uses a fixed-size static table
    Performs no syscall filtering
    Provides no tracing infrastructure
    Supports only interrupt-based syscall entry

These limitations are acceptable for the current development stage.

---

## 13. Future Enhancements

Planned improvements:

    Additional syscall arguments
    Syscall filtering
    Syscall tracing
    Syscall monitoring
    User-pointer validation
    Advanced dispatch policies

These features are not currently implemented.

---

## Contributor Notes

- Preserve dispatcher determinism.
- Keep dispatch() free of syscall-specific logic.
- Do not mutate the syscall table at runtime.
- Preserve ENOSYS behavior for unknown syscall numbers.
- Keep documentation synchronized with dispatcher.rs and table.rs.
- Update syscall harness coverage whenever dispatcher behavior changes.

---

## License

MIT or Apache 2.0 — TBD.