# Bulldog Kernel – Syscall Table (Current Implementation)

This document describes Bulldog’s *current* syscall table: the static mapping from syscall
numbers to syscall handler functions.  
It reflects the real behavior of:

- `syscall_table.rs` (static table)
- `dispatcher.rs` (lookup + dispatch)
- `syscall.rs` (entry path)
- `syscall-harness-guide.md` (validation)
- `privilege-switching.md` (Ring 3 → Ring 0 transition)

This is not a future design document — it describes exactly how Bulldog’s syscall table works today.

---

## 1. Overview

The syscall table is a **static, compile‑time defined mapping** from syscall numbers to
handler functions.  
It is intentionally simple and deterministic.

The table provides:

- A fixed ABI  
- Stable syscall numbering  
- A single source of truth for all syscalls  
- A contributor‑friendly workflow  
- A predictable dispatch path validated by the syscall harness

There is **no dynamic registration** and **no runtime mutation**.

---

## 2. Syscall Number Allocation (Current)

These are the syscall numbers currently implemented in Bulldog:

| Number | Name   | Handler Function           | Notes |
|--------|--------|----------------------------|-------|
| **1** | write  | `sys_write`                | FD write |
| **2** | exit   | `sys_exit_trampoline`      | Process exit (stub) |
| **3** | open   | `sys_open`                 | Returns FD |
| **4** | read   | `sys_read`                 | FD read |
| **5** | alloc  | `sys_alloc_trampoline`     | Allocates memory |
| **6** | free   | `sys_free_trampoline`      | Frees memory |
| **7** | close  | `sys_close_trampoline`     | Closes FD |

### Numbering invariants

- Numbers must be **unique**  
- Numbers must be **stable** across kernel versions  
- Numbers must be **documented** in `syscall.md`  
- Numbers must be **added only at the end** of the table  
- Numbers must **never be reused**  

These rules ensure ABI stability.

---

## 3. Syscall Table Structure

The table is defined in Rust as:

```rust
pub static SYSCALL_TABLE: SyscallTable = SyscallTable::new(&[
    (1, sys_write),
    (2, sys_exit_trampoline),
    (3, sys_open),
    (4, sys_read),
    (5, sys_alloc_trampoline),
    (6, sys_free_trampoline),
    (7, sys_close_trampoline),
]);
```

Each entry contains:

- A syscall number (`u64`)
- A handler function pointer (`SyscallFn`)

The table is stored in a static region and never modified.

---

## 4. Handler Function Signature

All syscall handlers use the same signature:

```rust
pub type SyscallFn = fn(args: SyscallArgs) -> u64;
```

Where:

```rust
pub struct SyscallArgs {
    pub arg0: u64,
    pub arg1: u64,
    pub arg2: u64,
}
```

This matches the dispatcher’s ABI:

| Register | Meaning |
|----------|---------|
| `rax` | syscall number |
| `rdi` | arg0 |
| `rsi` | arg1 |
| `rdx` | arg2 |

Return value is placed in `rax`.

---

## 5. Dispatcher Integration

The dispatcher performs:

```rust
match SYSCALL_TABLE.get(num) {
    Some(handler) => handler(args),
    None => SysError::InvalidSyscall.into(),
}
```

This ensures:

- Deterministic lookup  
- No dynamic mutation  
- No runtime registration  
- No fallback paths  
- No handler ambiguity  

The dispatcher is intentionally minimal.

---

## 6. Syscall Harness Integration

The syscall harness validates:

### Table correctness
- No missing entries  
- No duplicate numbers  
- No gaps  
- No incorrect ordering  

### Handler correctness
- Correct return value  
- Correct argument passing  
- Correct register preservation  
- Correct privilege return  

### Error paths
- Invalid syscall → `InvalidSyscall`  
- Unimplemented syscall → `NotImplemented`  

If the table is incorrect, the harness prints:

```
TABLE ERROR: syscall 7 missing handler
```

---

## 7. Adding a New Syscall

To add a new syscall:

### Step 1 — Add the handler
In `syscall/<name>.rs`:

```rust
pub fn sys_foo(args: SyscallArgs) -> u64 {
    // implementation
}
```

### Step 2 — Add the number
In `syscall/stubs.rs`:

```rust
pub const SYS_FOO: u64 = 8;
```

### Step 3 — Add the table entry
In `syscall_table.rs`:

```rust
(SYS_FOO, sys_foo),
```

### Step 4 — Document it
In `syscall.md`.

### Step 5 — Add harness tests
In the user‑mode test program.

### Step 6 — Validate
Run the syscall harness.

---

## 8. Error Handling

If a syscall number is not found:

```rust
SysError::InvalidSyscall
```

If a syscall is known but not implemented:

```rust
SysError::NotImplemented
```

Errors are encoded into `u64` and returned in `rax`.

---

## 9. Future Enhancements

Planned improvements:

- Extended ABI for large arguments  
- Per‑process syscall filtering  
- Syscall tracing  
- Syscall logging buffers  
- Fast‑path syscalls (no dispatcher)  
- VFS integration for `read`/`write`  
- Memory‑safe user pointer validation  

These will be documented once implemented.

---

## 10. Contributor Notes

- Add new syscalls only at the end of the table  
- Never reuse syscall numbers  
- Never mutate the table at runtime  
- Keep handler signatures consistent  
- Update `syscall.md` when adding new syscalls  
- Update the syscall harness when adding new syscalls  

---

## License

MIT or Apache 2.0 — TBD.
