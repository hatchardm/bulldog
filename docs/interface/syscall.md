# Bulldog Kernel – Syscall Interface (Current Implementation)

This document describes Bulldog’s *current* syscall interface: the ABI, calling convention,
numbering, handler signatures, and kernel entry/exit behavior.  
It reflects the real behavior of:

- `syscall.rs` (entry + dispatcher)
- `syscall_table.rs` (mapping)
- `dispatcher.md` (dispatch logic)
- `privilege-switching.md` (Ring 3 ↔ Ring 0)
- `syscall-harness-guide.md` (validation)

This is not a future design document — it describes exactly how Bulldog’s syscall interface works today.

---

## 1. Overview

Syscalls are the **only legal interface** between user mode (Ring 3) and kernel mode (Ring 0).

Bulldog’s syscall subsystem provides:

- A stable ABI  
- Deterministic register behavior  
- A static syscall table  
- A minimal dispatcher  
- A strict privilege‑switching path  
- A syscall harness for validation  

Syscalls are invoked using:

```
int SYSCALL_VECTOR
```

This vector is configured with DPL=3 so user mode can call it.

---

## 2. Syscall ABI (Current)

Bulldog uses a simple, Linux‑style ABI:

| Register | Meaning |
|----------|---------|
| `rax` | syscall number |
| `rdi` | arg0 |
| `rsi` | arg1 |
| `rdx` | arg2 |
| `r10` | arg3 (future) |
| `r8`  | arg4 (future) |
| `r9`  | arg5 (future) |

Return value is placed in:

```
rax
```

### ABI invariants

- Only `rax` may be modified  
- All other registers must be preserved  
- Flags must be preserved  
- Segment selectors must be preserved  
- User stack pointer must be preserved  

These invariants are enforced by the syscall harness.

---

## 3. Syscall Number Allocation (Current)

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

### Numbering rules

- Numbers must be **unique**  
- Numbers must be **stable**  
- Numbers must be **added only at the end**  
- Numbers must **never be reused**  
- All numbers must be documented here  

This ensures ABI stability.

---

## 4. Syscall Handler Signature

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

Handlers must:

- Follow the ABI  
- Not modify non‑argument registers  
- Return a `u64` result  
- Handle errors deterministically  

---

## 5. Kernel Entry Path (User → Kernel)

User mode enters the kernel via:

```
int SYSCALL_VECTOR
```

The CPU:

1. Checks IDT gate permissions (DPL=3)  
2. Switches to the kernel stack using TSS.rsp0  
3. Pushes a privilege‑transition frame  
4. Executes the syscall entry stub  
5. Calls `syscall_entry()` in Rust  

The dispatcher runs inside `syscall_entry()`.

See:  
`privilege-switching.md`  
`dispatcher.md`

---

## 6. Dispatcher Flow

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
- No fallback paths  
- No handler ambiguity  

The dispatcher is intentionally minimal.

---

## 7. Error Handling

Errors are returned using:

```rust
SysError
```

Common errors:

- `InvalidSyscall`  
- `BadAddress`  
- `PermissionDenied`  
- `NotImplemented`  

Errors are encoded into `u64` and returned in `rax`.

---

## 8. Kernel Return Path (Kernel → User)

Returning to user mode uses:

```
iretq
```

The return frame contains:

- User RIP  
- User CS (DPL=3)  
- User RFLAGS  
- User RSP  
- User SS (DPL=3)

The kernel must not corrupt this frame.

### Return invariants

- User RSP must be restored  
- User RIP must be restored  
- CS/SS must be Ring 3  
- Flags must be preserved  
- Non‑return registers must be preserved  

The syscall harness validates all of these.

---

## 9. Syscall Harness Integration

The syscall harness tests:

### ABI correctness
- Argument passing  
- Register preservation  
- Return value correctness  

### Privilege switching
- Correct kernel entry  
- Correct kernel exit  
- Correct `iretq` behavior  

### Table correctness
- No missing handlers  
- No duplicate numbers  
- No incorrect ordering  

### Error paths
- Invalid syscall → `InvalidSyscall`  
- Unimplemented syscall → `NotImplemented`  

See:  
`syscall-harness-guide.md`

---

## 10. Adding a New Syscall

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
In this file.

### Step 5 — Add harness tests
In the user‑mode test program.

### Step 6 — Validate
Run the syscall harness.

---

## 11. Future Enhancements

Planned improvements:

- Extended ABI for large arguments  
- Memory‑safe user pointer validation  
- Fast‑path syscalls (no dispatcher)  
- Per‑process syscall filtering  
- Syscall tracing  
- Syscall logging buffers  
- VFS integration for `read`/`write`  

These will be documented once implemented.

---

## 12. Contributor Notes

- Syscall numbers must remain stable  
- Add new syscalls only at the end  
- Never reuse syscall numbers  
- Keep handler signatures consistent  
- Update `syscall-table.md` when adding new syscalls  
- Update the syscall harness when adding new syscalls  

---

## License

MIT or Apache 2.0 — TBD.
