## Bulldog Kernel – Syscall Table (Current Implementation)

Reviewed: November 2026

This document describes Bulldog's current syscall table: the static mapping from syscall numbers to syscall handler functions.

It reflects the implementation found in:

    kernel/src/syscall/table.rs
    kernel/src/syscall/dispatcher.rs
    kernel/src/syscall/mod.rs
    kernel/src/tests/syscall_harness.rs

It complements:

    syscall.md
    dispatcher.md
    syscall-entry.md
    privilege-switching.md

This is not a future design document. It describes the syscall table currently used by the running kernel and validated by the syscall harness.

---

## 1. Overview

The syscall table is a static, compile-time mapping from syscall numbers to handler functions.

The table provides:

    Stable syscall numbering
    Deterministic dispatch
    Constant-time lookup
    A single source of truth
    Harness-validated behavior

There is:

    No dynamic registration
    No runtime mutation
    No runtime reordering

The dispatcher consults this table whenever user-mode code invokes:

    int 0x80

through the syscall subsystem.

---

## 2. Syscall Number Allocation (Current)

Currently implemented syscalls:

<table>
<tr>
<th>Number</th>
<th>Name</th>
<th>Handler Function</th>
<th>Notes</th>
</tr>

<tr>
<td>1</td>
<td>write</td>
<td>sys_write</td>
<td>FD write</td>
</tr>

<tr>
<td>2</td>
<td>exit</td>
<td>sys_exit_trampoline</td>
<td>Process exit and cleanup</td>
</tr>

<tr>
<td>3</td>
<td>open</td>
<td>sys_open</td>
<td>Returns file descriptor</td>
</tr>

<tr>
<td>4</td>
<td>read</td>
<td>sys_read</td>
<td>FD read</td>
</tr>

<tr>
<td>5</td>
<td>alloc</td>
<td>sys_alloc_trampoline</td>
<td>Allocates memory</td>
</tr>

<tr>
<td>6</td>
<td>free</td>
<td>sys_free_trampoline</td>
<td>Frees memory</td>
</tr>

<tr>
<td>7</td>
<td>close</td>
<td>sys_close_trampoline</td>
<td>Closes file descriptor</td>
</tr>

</table>

### Numbering Invariants

- Syscall numbers must be unique.
- Syscall numbers must remain stable.
- New syscall numbers must be added at the end of the allocation range.
- Existing syscall numbers must never be reused.
- ABI compatibility takes priority over numbering convenience.

These rules preserve syscall ABI stability.

---

## 3. Syscall Table Structure

The syscall subsystem exposes:

```rust
pub static SYSCALL_TABLE:
    [Option<SyscallFn>; SYSCALL_TABLE_SIZE];
```

Current table size:

```text
512 entries
```

Implemented syscall slots contain function pointers.

Unimplemented syscall slots contain:

```rust
None
```

The table is initialized at compile time and never modified at runtime.

---

## 4. Handler Function Signature

The syscall table stores handlers using:

```rust
pub type SyscallFn =
    fn(u64, u64, u64) -> u64;
```

Arguments correspond to:

```text
arg0
arg1
arg2
```

which originate from:

```text
rdi
rsi
rdx
```

during syscall dispatch.

Some handlers use trampoline functions to adapt specialized implementations to the common ABI.

Current trampoline handlers:

```text
sys_exit_trampoline
sys_alloc_trampoline
sys_free_trampoline
sys_close_trampoline
```

---

## 5. Dispatcher Integration

The dispatcher performs handler lookup through:

```rust
lookup(num)
```

Current logic:

```rust
match lookup(num) {
    Some(fun) => fun(arg0, arg1, arg2),
    None => err(errno::ENOSYS),
}
```

Properties:

    Deterministic lookup
    No dynamic registration
    No runtime mutation
    No handler ambiguity

Unknown syscall numbers return:

    ENOSYS

rather than generating an exception.

---

## 6. Syscall Harness Integration

The syscall harness validates syscall-table behavior alongside syscall functionality.

### Table Correctness

The harness verifies:

- Implemented syscalls resolve correctly
- Unimplemented syscalls return ENOSYS
- Number allocation remains stable
- Handlers remain reachable

### Handler Correctness

The harness verifies:

- Argument passing
- Return-value behavior
- Error propagation
- FD behavior
- Allocator behavior
- Process-exit behavior

### Dispatch Correctness

The harness also validates:

- Entry through vector 0x80
- Dispatcher execution
- Handler invocation
- Correct return values

The syscall harness serves as the primary behavioral validation source for the syscall subsystem.

---

## 7. Adding a New Syscall

### Step 1

Implement the syscall:

```text
kernel/src/syscall/<name>.rs
```

### Step 2

Allocate a syscall number:

```text
kernel/src/syscall/stubs.rs
```

Example:

```rust
pub const SYS_FOO: u64 = 8;
```

### Step 3

Register the syscall:

```text
kernel/src/syscall/table.rs
```

Example:

```rust
t[SYS_FOO as usize] = Some(sys_foo);
```

### Step 4

Update documentation:

```text
syscall.md
syscall-table.md
docs/interface/syscalls/<name>.md
```

### Step 5

Add syscall-harness coverage.

### Step 6

Validate with the syscall harness.

---

## 8. Error Handling

Unknown syscall numbers return:

```text
ENOSYS
```

Current syscall implementations may also return:

```text
EBADF
EFAULT
EINVAL
EMFILE
ENOMEM
```

Errors are encoded as negative errno values and returned through:

```text
rax
```

The syscall harness validates both success and failure paths.

---

## 9. Current Limitations

Current implementation:

    Supports three syscall arguments
    Uses a fixed-size syscall table
    Uses static handler registration
    Does not support dynamic loading
    Does not support syscall filtering

These limitations are acceptable for the current development stage.

---

## 10. Future Enhancements

Planned improvements:

    Additional syscall arguments
    User-pointer validation
    Syscall tracing
    Syscall filtering
    Expanded file operations
    Extended process-management syscalls

These capabilities are not currently implemented.

---

## Contributor Notes

- Never reuse syscall numbers.
- Add new syscall numbers only at the end of the allocation range.
- Do not mutate the syscall table at runtime.
- Keep syscall documentation synchronized with table.rs.
- Update the syscall harness whenever syscall behavior changes.
- Treat the syscall harness as the authoritative source of behavioral validation.

---

## License

MIT or Apache 2.0 — TBD.