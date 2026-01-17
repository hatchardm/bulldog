# Bulldog Syscall Architecture
A living document describing the current syscall ABI, the long‑term capability‑secured model, and the migration path between them.

======================================================================

1. Overview
-----------

Bulldog exposes a small, stable syscall interface used by early user‑mode code
and by the kernel’s syscall harness. The current implementation is intentionally
minimal: a synchronous `int 0x80` entry point, a static syscall table, and a
handful of core operations (read, write, open, close, alloc, free, exit).

This document describes:

- Current Implementation (v0.x) — the exact ABI and behavior implemented today
- Target Model (v1.x) — the capability‑secured, message‑based syscall architecture
- Migration Path — how Bulldog evolves without breaking contributors

======================================================================

2. Current Implementation (v0.x)
--------------------------------

This section documents the syscall subsystem *as it exists today in the Bulldog
kernel source tree*. It is the authoritative reference for contributors
implementing or modifying syscalls.

----------------------------------------------------------------------
2.1 Syscall Entry: `int 0x80`
----------------------------------------------------------------------

Bulldog uses the classic x86‑64 software interrupt mechanism:

    int 0x80

The IDT entry for vector `0x80` is configured with:

- DPL = 3 (callable from user mode)
- handler = `syscall_handler` (naked assembly)

Register ABI on entry:

- rax — syscall number
- rdi — argument 0
- rsi — argument 1
- rdx — argument 2

The handler calls:

    extern "C" fn rust_dispatch(num: u64, a0: u64, a1: u64, a2: u64) -> u64

Return value is placed in `rax` before `iretq`.

----------------------------------------------------------------------
2.2 Syscall Table
----------------------------------------------------------------------

Syscalls are stored in a static table:

    static SYSCALL_TABLE: [Option<SyscallFn>; 512]

Where:

    type SyscallFn = fn(u64, u64, u64) -> u64;

All syscalls must conform to this 3‑argument ABI.

Current syscall numbers:

| Number | Name  | Handler                 |
|--------|--------|-------------------------|
| 1      | write | sys_write               |
| 2      | exit  | sys_exit_trampoline     |
| 3      | open  | sys_open                |
| 4      | read  | sys_read                |
| 5      | alloc | sys_alloc_trampoline    |
| 6      | free  | sys_free_trampoline     |
| 7      | close | sys_close_trampoline    |

Unused entries return `-ENOSYS`.

----------------------------------------------------------------------
2.3 Error Model
----------------------------------------------------------------------

Bulldog uses Linux‑style errno values:

- Success: return a non‑negative u64
- Error: return `-(errno)` encoded as u64

Helpers:

    err(errno)
    err_from(Errno)

`errno.rs` defines the full errno set and `strerror()`.

----------------------------------------------------------------------
2.4 User Pointer Validation
----------------------------------------------------------------------

User pointers must:

- be canonical
- lie in the lower half (<= 0x0000_7FFF_FFFF_FFFF)
- be non‑null

Helpers:

- is_user_ptr(ptr)
- copy_from_user(src, dst)
- copy_to_user(dst, src)
- copy_cstr_from_user(ptr, buf)

Used by read, write, and open.

----------------------------------------------------------------------
2.5 File Descriptors
----------------------------------------------------------------------

Bulldog implements a simple FD table:

- Global Mutex<Option<FdTable>> (single‑process for now)
- FD 1 is Stdout
- Usable FD range: 3..=64
- fd_alloc, fd_get, fd_close manage entries

FdEntry contains:

- file: Box<dyn FileLike + Send>
- flags
- offset (unused for VFS; VFS uses internal offsets)

FileLike trait:

    trait FileLike {
        fn read(&mut self, buf) -> Result<usize, Errno>;
        fn write(&mut self, buf) -> Result<usize, Errno>;
        fn close(&mut self) -> Result<(), Errno>;
        fn seek(&mut self, offset) -> Result<(), Errno>;
    }

Default implementations return EBADF or EINVAL.

VFS files are wrapped in VfsFileLike.

----------------------------------------------------------------------
2.6 Implemented Syscalls
----------------------------------------------------------------------

write(fd, buf, len)
-------------------
- Validates FD
- Copies user buffer → kernel scratch buffer
- Calls FileLike::write
- Returns bytes written or -errno

read(fd, buf, len)
------------------
- Validates FD
- Reads into kernel scratch buffer
- Copies scratch buffer → user buffer
- Returns bytes read or -errno

open(path, flags, mode)
-----------------------
- Copies NUL‑terminated path from user
- If path starts with "/vfs/", resolves via VFS
- Calls FileOps::rewind() on each open
- Wraps file in VfsFileLike
- Allocates FD via fd_alloc

close(fd)
---------
- Removes FD from table
- Calls FileLike::close

alloc(size)
-----------
- Allocates memory via global allocator
- Returns pointer or -ENOMEM

free(ptr, size)
---------------
- Frees memory via global allocator

exit(code)
----------
- Logs exit code
- Clears FD table
- Returns (stub; no real process teardown yet)

For details on how the syscall layer is validated and tested, see:
`docs/syscall_harness_guide.md`.

======================================================================

3. Target Model (v1.x): Capability‑Secured Syscalls
---------------------------------------------------

The long‑term design replaces the synchronous, register‑based ABI with a
capability‑secured, message‑based syscall architecture.

Goals:

- Capability tokens
- SyscallMessage format
- Dispatcher validation
- Worker thread pool
- Audit logging
- Per‑process FD tables
- Memory capabilities
- Zero‑copy I/O
- Safe Rust wrapper library

SyscallMessage:

    struct SyscallMessage {
        num: u64,
        args: [u64; 3],
        capability: CapabilityToken,
        pid: u64,
        tid: u64,
        timestamp: u64,
    }

Capabilities describe:

- allowed operations
- resource identity
- rights
- lifetime and revocation

Worker threads execute syscalls asynchronously.

Wrapper library constructs messages and attaches capabilities.

======================================================================

4. Migration Path
-----------------

Phase 1 — Document and Stabilize (current)
- Document current ABI
- Add tracing and tests

Phase 2 — Introduce SyscallMessage
- Add message struct and queue
- Add dispatcher validation layer

Phase 3 — Add Capability Tokens
- Introduce capability types
- Wrap FD and memory resources

Phase 4 — Add Worker Thread Pool
- Move syscall execution off interrupt path
- Add structured responses and audit logging

Phase 5 — Replace Raw Pointers and FDs
- Memory capabilities
- Capability‑bound handles
- Real process teardown

Phase 6 — Deprecate int 0x80
- Introduce syscall/sysret fast path
- Keep int 0x80 for compatibility

======================================================================

5. Contributing New Syscalls
----------------------------

Until the capability model is implemented:

1. Add syscall number in stubs.rs
2. Add handler to table.rs
3. Implement syscall in its own module
4. Use:
   - fd_get, fd_alloc, fd_close
   - copy_from_user, copy_to_user
   - err, err_from
5. Add logging
6. Add tests under feature = "syscall_tests"

======================================================================

6. Appendix: Current Syscall ABI Summary
----------------------------------------

Entry mechanism:
    int 0x80

Registers:
    rax = num, rdi = arg0, rsi = arg1, rdx = arg2

Return:
    rax = result or -(errno)

Table size:
    512 entries

Implemented syscalls:
    write, read, open, close, alloc, free, exit

======================================================================

End of Document