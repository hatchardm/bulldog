# Bulldog Syscall Architecture  
*A living document describing the current syscall ABI, the long‑term capability‑secured model, and the migration path between them.*

---

## 1. Overview

Bulldog exposes a small, stable syscall interface used by early user‑mode code and kernel tests.  
The current implementation is intentionally minimal: a synchronous `int 0x80` entry point, a static syscall table, and a handful of core operations (read, write, open, close, alloc, free, exit).

This document describes:

- **Current Implementation (v0.x)** — the exact ABI and behavior implemented today  
- **Target Model (v1.x)** — the capability‑secured, message‑based syscall architecture Bulldog is evolving toward  
- **Migration Path** — how we get from the current model to the target model without breaking contributors or user‑mode code  

---

# 2. Current Implementation (v0.x)

This section documents the syscall subsystem *as it exists today in the Bulldog kernel source tree*.  
It is the authoritative reference for contributors implementing or modifying syscalls.

---

## 2.1 Syscall Entry: `int 0x80`

Bulldog uses the classic x86‑64 software interrupt mechanism:

- User code triggers:  
  ```
  int 0x80
  ```
- The IDT entry for vector `0x80` is configured with:
  - DPL = 3 (callable from user mode)
  - handler = `syscall_handler` (naked assembly)

### Register ABI

On entry:

- `rax` — syscall number  
- `rdi` — argument 0  
- `rsi` — argument 1  
- `rdx` — argument 2  

The handler preserves callee‑saved registers, shuffles arguments, and calls:

```rust
extern "C" fn rust_dispatch(num: u64, a0: u64, a1: u64, a2: u64) -> u64
```

The return value is placed in `rax` before `iretq`.

---

## 2.2 Syscall Table

Syscalls are stored in a static table:

```rust
static SYSCALL_TABLE: [Option<SyscallFn>; 512]
```

Where:

```rust
type SyscallFn = fn(u64, u64, u64) -> u64;
```

All syscalls must conform to this 3‑argument ABI.  
1‑argument syscalls use trampolines.

### Current syscall numbers

| Number | Name      | Handler                     |
|--------|-----------|-----------------------------|
| 1      | write     | `sys_write`                 |
| 2      | exit      | `sys_exit_trampoline`       |
| 3      | open      | `sys_open`                  |
| 4      | read      | `sys_read`                  |
| 5      | alloc     | `sys_alloc_trampoline`      |
| 6      | free      | `sys_free_trampoline`       |
| 7      | close     | `sys_close_trampoline`      |

Unused entries are `None` and return `-ENOSYS`.

---

## 2.3 Error Model

Bulldog uses Linux‑style errno values:

- Success: return a non‑negative `u64`
- Error: return `-(errno)` encoded as a `u64`

Helpers:

```rust
err(errno)        // encode numeric errno
err_from(Errno)   // encode typed Errno
```

`errno.rs` defines the full errno set and a `strerror()` mapping.

---

## 2.4 User Pointer Validation

User pointers must:

- be canonical  
- lie in the lower half (`<= 0x0000_7FFF_FFFF_FFFF`)  
- be non‑null  

Helpers:

- `is_user_ptr(ptr)`  
- `copy_from_user(src, dst)`  
- `copy_to_user(dst, src)`  
- `copy_cstr_from_user(ptr, buf)`  

These functions are used by `read`, `write`, and `open`.

---

## 2.5 File Descriptors

Bulldog implements a simple FD table:

- Global `Mutex<Option<FdTable>>` (single‑process for now)
- FD 1 is initialized as `Stdout`
- Usable FD range: `3..=64`
- `fd_alloc`, `fd_get`, `fd_close` manage entries
- `FdEntry` contains:
  - `file: Box<dyn FileLike + Send>`
  - `flags`
  - `offset`

### FileLike trait

```rust
trait FileLike {
    fn read(&mut self, buf) -> Result<usize, Errno>;
    fn write(&mut self, buf) -> Result<usize, Errno>;
    fn close(&mut self) -> Result<(), Errno>;
    fn seek(&mut self, offset) -> Result<(), Errno>;
}
```

Default implementations return `EBADF` or `EINVAL`.

`Stdout` is the only built‑in implementation today.

---

## 2.6 Implemented Syscalls

### `write(fd, buf, len)`
- Validates FD via `fd_get`
- Copies user buffer → kernel scratch buffer
- Calls `FileLike::write`
- Returns bytes written or `-errno`

### `read(fd, buf, len)`
- Validates FD
- Reads device → kernel scratch buffer
- Copies scratch buffer → user buffer
- Returns bytes read or `-errno`

### `open(path, flags, mode)`
- Copies NUL‑terminated path from user
- Currently always returns a `Stdout` handle
- Allocates FD via `fd_alloc`

### `close(fd)`
- Removes FD from table
- Calls `FileLike::close`

### `alloc(size)`
- Allocates memory via Rust global allocator
- Returns pointer or `-ENOMEM`

### `free(ptr, size)`
- Frees memory via Rust global allocator

### `exit(code)`
- Logs exit code
- Clears FD table
- Returns (stub; real process teardown not implemented)

---

# 3. Target Model (v1.x): Capability‑Secured Syscalls

The long‑term design replaces the synchronous, register‑based ABI with a **capability‑secured, message‑based syscall architecture**.

This section describes the intended model.

---

## 3.1 Goals

- Strong security via **capability tokens**
- Uniform **SyscallMessage** format
- **Dispatcher** validates capabilities and arguments
- **Worker thread pool** executes syscalls asynchronously
- **Audit logging** for every syscall
- **Per‑process FD tables** with capability‑bound handles
- **Memory capabilities** instead of raw pointers
- **Zero‑copy I/O** where possible
- **User‑space wrapper library** providing safe Rust APIs

---

## 3.2 SyscallMessage

All syscalls become structured messages:

```rust
struct SyscallMessage {
    num: u64,
    args: [u64; 3],
    capability: CapabilityToken,
    pid: u64,
    tid: u64,
    timestamp: u64,
}
```

Messages are placed into a syscall queue.

---

## 3.3 Capability Tokens

Every syscall requires a capability token describing:

- allowed operations (read/write/seek/open/close/alloc/free/exit)
- resource identity (FD, memory region, path)
- rights (read/write/execute)
- lifetime and revocation

Capabilities are validated by the dispatcher before a syscall is enqueued.

---

## 3.4 Worker Thread Pool

Syscalls are executed by kernel worker threads:

- dequeue message  
- validate arguments  
- call handler  
- produce structured response  
- log audit entry  

This enables:

- concurrency  
- isolation  
- scheduling fairness  
- future async I/O  

---

## 3.5 Wrapper Library

User‑space code will not call `int 0x80` directly.  
Instead, it will use a safe Rust API:

```rust
let fd = bulldog::open("/dev/tty", OpenFlags::Write)?;
bulldog::write(fd, b"hello")?;
```

The wrapper library:

- constructs `SyscallMessage`
- attaches capability tokens
- handles responses
- provides ergonomic Rust types

---

# 4. Migration Path

This section describes how Bulldog evolves from the current model to the target model without breaking contributors.

---

## Phase 1 — Document and Stabilize (current)

- Document current ABI (this file)
- Ensure all syscalls follow consistent patterns
- Add syscall tracing and logging
- Add tests for existing syscalls

---

## Phase 2 — Introduce SyscallMessage

- Add message struct
- Add syscall queue
- Add dispatcher validation layer
- Keep existing synchronous handlers as worker functions

---

## Phase 3 — Add Capability Tokens

- Introduce capability types
- Wrap FD entries in capability metadata
- Wrap memory regions in capabilities
- Update dispatcher to enforce capability checks

---

## Phase 4 — Add Worker Thread Pool

- Move syscall execution off the interrupt path
- Add worker threads
- Add structured responses
- Add audit logging

---

## Phase 5 — Replace Raw Pointers and FDs

- Replace raw pointers with memory capabilities
- Replace numeric FDs with capability‑bound handles
- Introduce real VFS
- Implement real process teardown

---

## Phase 6 — Deprecate `int 0x80`

- Introduce `syscall/sysret` fast path
- Keep `int 0x80` for compatibility
- Migrate wrapper library to new entry mechanism

---

# 5. Contributing New Syscalls

Until the capability model is implemented, contributors should:

1. Add a syscall number in `stubs.rs`
2. Add the handler to `table.rs`
3. Implement the syscall in its own module
4. Use:
   - `fd_get`, `fd_alloc`, `fd_close`
   - `copy_from_user`, `copy_to_user`
   - `err`, `err_from`
5. Add logging
6. Add tests under `feature = "syscall_tests"`

When the capability model lands, this section will be updated.

---

# 6. Appendix: Current Syscall ABI Summary

### Entry mechanism  
`int 0x80`

### Registers  
`rax = num`, `rdi = arg0`, `rsi = arg1`, `rdx = arg2`

### Return  
`rax = result or -(errno)`

### Table size  
512 entries

### Implemented syscalls  
write, read, open, close, alloc, free, exit

---

# End of Document