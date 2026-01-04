# Bulldog Kernel – Syscall Harness Guide

This document describes Bulldog’s syscall test harness: the infrastructure used to validate
syscall entry, dispatch, register preservation, and return‑path correctness in the
`feature/syscall` branch.

It complements `syscall.md` and `privilege-switching.md` by focusing specifically on
testing, logging, and contributor workflows.

---

## Purpose

The syscall harness provides:

- A controlled environment for invoking syscalls from user mode  
- Logging of entry/exit metadata  
- Register preservation checks  
- Error‑path validation  
- A template for contributors adding new syscalls  
- A reproducible test loop for QEMU  

The harness ensures that every syscall added to Bulldog is:

- Correctly wired  
- Safe  
- Logged  
- Reproducible  
- Contributor‑friendly  

---

## Architecture Overview

The syscall harness consists of four cooperating components:

1. **User‑Mode Test Program**  
   A minimal Ring 3 program that triggers syscalls using `int 0x80`.

2. **Syscall Trampoline**  
   The privilege‑switching entry point that saves user context and enters the kernel.

3. **Dispatcher**  
   Routes syscall numbers to handlers via the syscall table.

4. **Logging Pipeline**  
   Emits structured logs for entry, exit, and error conditions.

The harness validates both the *control flow* and the *data flow* of syscalls.

---

## Harness Components

### 1. User‑Mode Test Program

A minimal test program should:

- Set up arguments  
- Trigger a syscall  
- Validate the return value  
- Print or log results  

Example outline:

```rust
pub extern "C" fn user_test() {
    unsafe {
        core::arch::asm!(
            "mov rax, 1",      // syscall number
            "mov rdi, 1",      // fd
            "mov rsi, msg",    // buffer
            "mov rdx, 5",      // length
            "int 0x80",
            msg = sym MESSAGE,
        );
    }
}
```

This program is linked into the user‑mode ELF used by QEMU.

---

### 2. Syscall Trampoline (Kernel Entry)

The trampoline:

- Saves user registers  
- Saves user RIP/RSP/CS/SS/RFLAGS  
- Switches to the kernel stack  
- Calls the dispatcher  
- Restores registers  
- Returns via `iretq`  

Logging occurs at both entry and exit.

---

### 3. Dispatcher

The dispatcher receives:

- syscall number  
- up to 6 arguments  
- user context  

It then performs:

- table lookup  
- handler invocation  
- error fallback for unknown syscalls  

---

### 4. Logging Pipeline

The harness requires consistent log formatting.

Entry log:

```
[SYSCALL] entry num=1 rdi=1 rsi=0x1000 rdx=5
```

Exit log:

```
[SYSCALL] exit num=1 ret=5
```

Error log:

```
[SYSCALL] unknown num=99
```

These logs are consumed by contributors and automated test scripts.

---

## Adding a New Syscall Test

To add a new syscall test:

1. **Assign a syscall number**  
   Update `stubs.rs` and `syscall-table.md`.

2. **Implement the stub**  
   Add a handler function with signature:  
   `fn(u64, u64, u64) -> u64`.

3. **Extend the syscall table**  
   Add a match arm or table entry.

4. **Add a user‑mode test case**  
   Modify the test program to invoke the new syscall.

5. **Define expected output**  
   Add entry/exit logs to the test harness documentation.

6. **Run under QEMU**  
   Validate logs and return values.

7. **Document the syscall**  
   Update `docs/syscall.md`.

---

## Expected Output Format

All syscall tests must follow this format:

```
[SYSCALL] entry num=<id> rdi=<arg1> rsi=<arg2> rdx=<arg3>
<optional handler logs>
[SYSCALL] exit num=<id> ret=<value>
```

Unknown syscalls must produce:

```
[SYSCALL] unknown num=<id>
```

---

## Register Preservation Tests

The harness verifies that:

- Caller‑saved registers (`rax`, `rcx`, `rdx`, `rsi`, `rdi`, `r8–r11`)  
  may be clobbered by the syscall.

- Callee‑saved registers (`rbx`, `rbp`, `r12–r15`)  
  must be preserved.

- `RFLAGS` must not leak privileged bits.

- User `RSP` must be restored exactly.

- Kernel `RSP` must remain aligned to 16 bytes.

A dedicated test case should snapshot registers before and after the syscall.

---

## Error Injection Tests

The harness includes tests for:

- Unknown syscall numbers  
- Invalid pointers  
- Invalid file descriptors  
- Misaligned user stacks  
- Guard page faults  
- Null arguments  

Each test must produce a deterministic log entry.

---

## User‑Mode Test Template

A minimal template for contributors:

```rust
pub extern "C" fn test_syscall(num: u64, a: u64, b: u64, c: u64) -> u64 {
    let ret: u64;
    unsafe {
        core::arch::asm!(
            "mov rax, {0}",
            "mov rdi, {1}",
            "mov rsi, {2}",
            "mov rdx, {3}",
            "int 0x80",
            out("rax") ret,
            in(reg) num, in(reg) a, in(reg) b, in(reg) c,
        );
    }
    ret
}
```

Contributors can call this helper from user‑mode tests.

---

## Contributor Tasks

- Add new syscall tests  
- Maintain consistent log formatting  
- Validate register preservation  
- Add expected output to documentation  
- Keep tests minimal and deterministic  
- Run all tests under QEMU before submitting PRs  

---

## Roadmap Context

- [x] Syscall table  
- [x] Dispatcher  
- [x] Logging  
- [ ] Full syscall harness  
- [ ] Automated test runner  
- [ ] User‑mode test suite  
- [ ] Process scheduler integration  

---

## License

MIT or Apache 2.0 — TBD.

