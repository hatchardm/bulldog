# 📖 Bulldog Kernel – Syscall Table Example

This document shows how Bulldog wires syscall numbers to stub functions.  
Contributors can use this as a reference when adding new syscalls.

---

## 🔢 Syscall Numbers

- **1 → `sys_write`**  
- **2 → `sys_exit`**  
- **3 → `sys_open`**  
- **4 → `sys_read`** (stub example, not yet implemented)

---

## 🛠 Table Implementation

```rust
// File: kernel/src/syscall/table.rs

use crate::syscall::stubs::{
    SYS_WRITE, SYS_EXIT, SYS_OPEN, SYS_READ,
    sys_write, sys_exit, sys_open, sys_read,
    SyscallFn,
};

/// Lookup table mapping syscall numbers to stub functions.
pub fn lookup(num: u64) -> Option<SyscallFn> {
    match num {
        SYS_WRITE => Some(sys_write),
        SYS_EXIT  => Some(sys_exit),
        SYS_OPEN  => Some(sys_open),
        SYS_READ  => Some(sys_read),
        _ => None,
    }
}

/// Fallback for unknown syscalls.
pub fn unknown(num: u64) -> u64 {
    log::warn!("Unknown syscall num={} invoked", num);
    u64::MAX // error code
}
```

---

## 🧪 Expected Output

When running with `--features syscall_tests`, a successful `sys_write` should produce:

```
[INFO] Syscall handler initialized at vector 0x80
[INFO] Syscall ready
[INFO] sys_write (fd=1, ptr=0x1, len=1)
hello bulldog
[INFO] syscall num=1 ret=0
```

---

## 📑 Stub Template Example – `sys_read`

To add a new syscall such as `sys_read`, follow these steps:

### 1. Define the constant in `stubs.rs`

```rust
// File: kernel/src/syscall/stubs.rs

pub const SYS_READ: u64 = 4; // next available syscall number

/// Syscall function signature: (arg1, arg2, arg3) -> u64
pub type SyscallFn = fn(u64, u64, u64) -> u64;

/// Example stub for sys_read
pub fn sys_read(fd: u64, ptr: u64, len: u64) -> u64 {
    log::info!(
        "sys_read (fd={}, ptr={:#x}, len={})",
        fd, ptr, len
    );

    // For now, return 0 to indicate success.
    // Later, implement actual read semantics in read.rs.
    0
}
```

### 2. Extend the lookup table in `table.rs`

```rust
// File: kernel/src/syscall/table.rs

use crate::syscall::stubs::{
    SYS_WRITE, SYS_EXIT, SYS_OPEN, SYS_READ,
    sys_write, sys_exit, sys_open, sys_read,
    SyscallFn,
};

pub fn lookup(num: u64) -> Option<SyscallFn> {
    match num {
        SYS_WRITE => Some(sys_write),
        SYS_EXIT  => Some(sys_exit),
        SYS_OPEN  => Some(sys_open),
        SYS_READ  => Some(sys_read),
        _ => None,
    }
}
```

### 3. Document expected behavior

- **sys_read(fd, ptr, len)** should attempt to read `len` bytes from file descriptor `fd` into buffer at `ptr`.  
- For now, the stub logs the call and returns `0`.  
- Once `read.rs` is implemented, update the stub to delegate to the real handler.

---

## 🧪 Example Output (stub mode)

When running with `--features syscall_tests` and invoking `sys_read`:

```
[INFO] sys_read (fd=0, ptr=0x1000, len=16)
[INFO] syscall num=4 ret=0
```

---

## 📊 Syscall Number Allocation Table

| Number | Name       | Status        | Notes                          |
|--------|------------|---------------|--------------------------------|
| 1      | sys_write  | Implemented   | Writes buffer to fd            |
| 2      | sys_exit   | Implemented   | Terminates process             |
| 3      | sys_open   | Implemented   | Opens file descriptor          |
| 4      | sys_read   | Stub example  | Reads buffer (to be implemented) |
| 5–15   | Reserved   | Future use    | Suggested for core POSIX calls |
| 16–31  | Reserved   | Future use    | Extended Bulldog syscalls       |
| 32+    | Experimental | Contributor proposals | Document in `docs/syscall.md` |

---

## 🤝 Contributor Notes

- **Add new syscall numbers** as constants in `stubs.rs`.  
- **Implement the stub function** with the signature `(u64, u64, u64) -> u64`.  
- **Extend the lookup match** in `table.rs` to route the new number.  
- **Document expected behavior** in `docs/syscall.md` so others can follow.  
- **Logging hygiene:** Ensure each syscall logs entry, arguments, and return value for contributor clarity.  
- **Error handling:** Unknown syscalls should always route through `unknown()` to avoid silent failures.  
- **Number allocation:** Use the table above to avoid collisions and keep numbering consistent.

---

## 🧭 Roadmap Context

This table provides a clear baseline for expanding Bulldog’s syscall harness.  
It is the foundation for building out the syscall dispatcher and eventually user‑mode execution.
