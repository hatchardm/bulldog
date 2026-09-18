# Bulldog Kernel – Syscall Dispatcher (Current Implementation)

This document describes Bulldog’s *current* syscall dispatcher architecture.  
It reflects the real behavior of:

- `syscall.rs` (entry point + dispatcher)
- `syscall_table.rs` (syscall number → handler mapping)
- `privilege-switching.md` (Ring 3 → Ring 0 transition)
- `syscall-harness-guide.md` (testing and validation)

This is not a future design document — it describes exactly how Bulldog’s dispatcher works today.

---

## 1. Overview

Bulldog’s syscall dispatcher is responsible for:

- Receiving control from user mode  
- Validating the syscall number  
- Dispatching to the correct handler  
- Preserving registers according to the syscall ABI  
- Returning values to user mode  
- Ensuring a clean `iretq` return path  

The dispatcher is intentionally minimal and deterministic.

---

## 2. Syscall Entry Path

User mode enters the kernel through the syscall vector:

```
SYSCALL_VECTOR
```

This vector is:

- Installed by the syscall subsystem  
- **Not** installed by `interrupts::init_idt()`  
- Configured with DPL=3 so user mode can invoke it  
- Routed onto the kernel stack via TSS  

The entry stub:

1. Saves user registers  
2. Switches to the kernel stack (TSS)  
3. Calls `syscall_entry()` in Rust  
4. Restores registers  
5. Returns via `iretq`

The dispatcher runs inside `syscall_entry()`.

---

## 3. Syscall ABI (Current)

Bulldog uses a simple, Linux‑style ABI:

| Register | Meaning |
|----------|---------|
| `rax` | syscall number |
| `rdi` | arg0 |
| `rsi` | arg1 |
| `rdx` | arg2 |
| `r10` | arg3 |
| `r8`  | arg4 |
| `r9`  | arg5 |

Return value is placed in:

```
rax
```

This ABI is enforced by the syscall harness.

---

## 4. Dispatcher Flow

The dispatcher is implemented in Rust:

```rust
pub fn syscall_dispatch(num: u64, args: SyscallArgs) -> SyscallResult {
    match SYSCALL_TABLE.get(num) {
        Some(handler) => handler(args),
        None => SyscallResult::error(SysError::InvalidSyscall),
    }
}
```

### Steps:

1. Extract syscall number from `rax`  
2. Extract arguments from registers  
3. Look up handler in `SYSCALL_TABLE`  
4. If found → call handler  
5. If not found → return `InvalidSyscall`  
6. Place return value in `rax`  
7. Return to user mode via `iretq`

The dispatcher itself is intentionally tiny.

---

## 5. Syscall Table

The syscall table is defined in `syscall_table.rs`:

```rust
pub static SYSCALL_TABLE: SyscallTable = SyscallTable::new(&[
    (0, sys_write),
    (1, sys_read),
    (2, sys_exit),
    // ...
]);
```

Properties:

- Deterministic ordering  
- No dynamic registration  
- No runtime mutation  
- Contributors add new syscalls here  

The syscall harness validates:

- Table ordering  
- Handler correctness  
- Register preservation  
- Return‑path correctness  

---

## 6. Error Handling

Errors are returned using:

```rust
SysError
```

Common errors:

- `InvalidSyscall`  
- `BadAddress`  
- `PermissionDenied`  
- `NotImplemented`  

Errors are encoded into `SyscallResult` and returned in `rax`.

---

## 7. Register Preservation

The dispatcher must preserve:

- All non‑return registers  
- Flags  
- Segment selectors  
- User stack pointer  

The syscall harness tests:

- That registers not used for arguments remain unchanged  
- That `rax` contains the correct return value  
- That `iretq` restores user mode correctly  

This is one of Bulldog’s strongest invariants.

---

## 8. Return Path

Returning to user mode uses:

```
iretq
```

The return frame contains:

- User RIP  
- User CS (DPL=3)  
- RFLAGS  
- User RSP  
- User SS (DPL=3)  

The dispatcher must not corrupt this frame.

---

## 9. Future Enhancements

Planned improvements:

- Per‑process syscall filtering  
- Syscall tracing  
- Syscall logging buffers  
- Fast‑path syscalls (no dispatcher)  
- Extended ABI for large arguments  
- Memory‑safe user pointer validation  
- VFS integration for `read`/`write`  

These will be documented once implemented.

---

## 10. Contributor Notes

- Add new syscalls only in `syscall_table.rs`  
- Keep dispatcher minimal and deterministic  
- Do not mutate the syscall table at runtime  
- Ensure all handlers follow the syscall ABI  
- Document new syscalls in `syscall.md`  
- Update the syscall harness when adding new syscalls  

---

## License

MIT or Apache 2.0 — TBD.
