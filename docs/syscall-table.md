# Bulldog Kernel – Syscall Table

This document describes how Bulldog wires syscall numbers to their handlers, how the static syscall table is initialized, and how contributors should add new syscalls.  
It reflects the **current, real implementation** in `kernel/src/syscall/table.rs`, not the older stub‑based example.

---

# 1. Overview

Bulldog uses a **static syscall table** with **512 entries**, indexed directly by syscall number.  
Each entry is either:

- `Some(handler_fn)` — a valid syscall handler  
- `None` — unimplemented, returns `-ENOSYS`  

All syscall handlers use the uniform 3‑argument ABI:

```rust
fn(u64, u64, u64) -> u64
```

1‑argument syscalls (e.g., `exit`, `alloc`, `free`, `close`) use **trampolines** that adapt the 3‑argument ABI to the real function signature.

---

# 2. Syscall Number Allocation

These are the syscall numbers currently implemented in Bulldog:

| Number | Name   | Handler Function           | Notes |
|--------|--------|----------------------------|-------|
| 1      | write  | `sys_write`                | FD write |
| 2      | exit   | `sys_exit_trampoline`      | Process exit (stub) |
| 3      | open   | `sys_open`                 | Returns FD |
| 4      | read   | `sys_read`                 | FD read |
| 5      | alloc  | `sys_alloc_trampoline`     | Allocates memory |
| 6      | free   | `sys_free_trampoline`      | Frees memory |
| 7      | close  | `sys_close_trampoline`     | Closes FD |

All other entries (8–511) are currently unassigned.

---

# 3. Static Table Implementation

The real syscall table lives in `kernel/src/syscall/table.rs` and looks like this:

```rust
// File: kernel/src/syscall/table.rs

use crate::syscall::stubs::*;
use crate::syscall::write::sys_write;
use crate::syscall::read::sys_read;
use crate::syscall::open::sys_open;
use crate::syscall::close::sys_close_trampoline;
use crate::syscall::exit::sys_exit_trampoline;
use crate::syscall::alloc::sys_alloc_trampoline;
use crate::syscall::free::sys_free_trampoline;

pub static SYSCALL_TABLE: [Option<SyscallFn>; 512] = {
    let mut table: [Option<SyscallFn>; 512] = [None; 512];

    table[SYS_WRITE as usize] = Some(sys_write);
    table[SYS_EXIT  as usize] = Some(sys_exit_trampoline);
    table[SYS_OPEN  as usize] = Some(sys_open);
    table[SYS_READ  as usize] = Some(sys_read);
    table[SYS_ALLOC as usize] = Some(sys_alloc_trampoline);
    table[SYS_FREE  as usize] = Some(sys_free_trampoline);
    table[SYS_CLOSE as usize] = Some(sys_close_trampoline);

    table
};
```

This table is constructed at compile time and never mutated at runtime.

---

# 4. Syscall Dispatch Path

The syscall entry point (`int 0x80`) eventually calls:

```rust
rust_dispatch(num, a0, a1, a2)
```

Which performs:

1. Bounds check (`num < 512`)
2. Table lookup
3. If entry is `None`: return `-ENOSYS`
4. Otherwise call the handler

Handlers return:

- `>= 0` on success  
- `-(errno)` on failure  

---

# 5. Trampolines

Some syscalls take fewer than 3 arguments.  
To maintain a uniform ABI, Bulldog uses trampolines:

Example: `sys_exit(code: u64)`

```rust
pub fn sys_exit_trampoline(code: u64, _a1: u64, _a2: u64) -> u64 {
    sys_exit(code)
}
```

This keeps the syscall table simple and consistent.

---

# 6. Adding a New Syscall

To add a new syscall:

### Step 1 — Assign a number in `stubs.rs`

```rust
pub const SYS_FOO: u64 = 8;
```

### Step 2 — Implement the handler

Create `kernel/src/syscall/foo.rs`:

```rust
pub fn sys_foo(a0: u64, a1: u64, a2: u64) -> u64 {
    // implementation
}
```

If your syscall takes fewer than 3 arguments, add a trampoline.

### Step 3 — Register it in the table

In `table.rs`:

```rust
table[SYS_FOO as usize] = Some(sys_foo);
```

### Step 4 — Document it

Add an entry to:

- `docs/syscall.md`  
- `docs/syscall-table.md`  

---

# 7. Syscall Number Allocation Table

| Range     | Purpose                         |
|-----------|---------------------------------|
| 1–15      | Core Bulldog syscalls           |
| 16–31     | Extended kernel syscalls        |
| 32–255    | Future VFS, process, IPC        |
| 256–511   | Experimental / contributor use  |

Contributors should avoid collisions by updating this table when adding new syscalls.

---

# 8. Example Logging Output

```
[INFO] syscall: num=1 write(fd=1, ptr=0x1000, len=5)
hello
[INFO] syscall: num=1 ret=5
```

---

# 9. Summary

- Bulldog uses a **static 512‑entry syscall table**  
- All syscalls use a **uniform 3‑argument ABI**  
- **Trampolines** adapt 1‑argument syscalls  
- Unimplemented syscalls return **-ENOSYS**  
- Contributors must update both the table and documentation  

This document is the authoritative reference for syscall number allocation and table wiring.

---

# End of Document