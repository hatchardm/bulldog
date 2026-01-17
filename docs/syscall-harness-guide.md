# Bulldog Kernel – Syscall Harness Guide
A companion document to `syscalls.md` describing Bulldog’s syscall test harness:
the infrastructure used to validate syscall entry, dispatch, register preservation,
and return‑path correctness.

======================================================================

1. Purpose
----------

The syscall harness provides a controlled environment for invoking syscalls from
user mode and validating:

- entry and exit paths
- register preservation
- error‑path behavior
- FD semantics
- VFS integration
- ABI stability

It ensures every syscall added to Bulldog is:

- correctly wired
- safe
- logged
- reproducible
- contributor‑friendly

======================================================================

2. Architecture Overview
------------------------

The harness consists of four cooperating components:

1. User‑Mode Test Program  
   A minimal Ring 3 program that triggers syscalls using `int 0x80`.

2. Syscall Trampoline  
   The privilege‑switching entry point that saves user context and enters the kernel.

3. Dispatcher  
   Routes syscall numbers to handlers via the syscall table.

4. Logging Pipeline  
   Emits structured logs for entry, exit, and error conditions.

The harness validates both control flow and data flow of syscalls.

======================================================================

3. Harness Components
---------------------

### 3.1 User‑Mode Test Program

A minimal test program:

- sets up arguments
- triggers a syscall
- validates the return value
- prints or logs results

Example outline:

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

This program is linked into the user‑mode ELF used by QEMU.

---

### 3.2 Syscall Trampoline (Kernel Entry)

The trampoline:

- saves user registers
- saves RIP/RSP/CS/SS/RFLAGS
- switches to the kernel stack
- calls the dispatcher
- restores registers
- returns via `iretq`

Logging occurs at both entry and exit.

---

### 3.3 Dispatcher

The dispatcher receives:

- syscall number
- up to 3 arguments
- user context

It performs:

- table lookup
- handler invocation
- fallback for unknown syscalls

---

### 3.4 Logging Pipeline

Entry log:

    [SYSCALL] entry num=1 rdi=1 rsi=0x1000 rdx=5

Exit log:

    [SYSCALL] exit num=1 ret=5

Unknown syscall:

    [SYSCALL] unknown num=99

======================================================================

4. Register Preservation Tests
------------------------------

The harness verifies:

- Caller‑saved registers (rax, rcx, rdx, rsi, rdi, r8–r11) may be clobbered.
- Callee‑saved registers (rbx, rbp, r12–r15) must be preserved.
- RFLAGS must not leak privileged bits.
- User RSP must be restored exactly.
- Kernel RSP must remain 16‑byte aligned.

A dedicated test snapshots registers before and after the syscall.

======================================================================

5. Error Injection Tests
------------------------

The harness includes tests for:

- unknown syscall numbers
- invalid pointers
- invalid file descriptors
- misaligned user stacks
- guard page faults
- null arguments

Each test must produce deterministic logs.

======================================================================

6. User‑Mode Test Template
--------------------------

A reusable helper for contributors:

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

======================================================================

7. Contributor Workflow
-----------------------

To add a new syscall test:

1. Assign a syscall number in `stubs.rs`.
2. Add the handler to the syscall table.
3. Implement the syscall in its own module.
4. Add a user‑mode test case.
5. Define expected log output.
6. Run under QEMU and validate behavior.
7. Update `docs/syscalls.md`.

======================================================================

8. Harness Invariants
---------------------

The harness guarantees:

- syscall ABI stability
- correct error encoding
- correct FD semantics
- correct VFS integration
- correct pointer validation
- correct register preservation
- correct return‑path behavior
- deterministic logging

Any change that breaks these invariants must be rejected.

======================================================================

9. Roadmap Context
------------------

- [x] Syscall table
- [x] Dispatcher (current synchronous version)
- [x] Logging
- [x] Full syscall harness
- [ ] Automated test runner
- [ ] User‑mode test suite
- [ ] Process scheduler integration
# Syscall Harness Contributor Checklist

Before submitting a PR that touches the syscall layer:

- [ ] All syscall harness tests pass under QEMU
- [ ] New syscalls have user‑mode tests
- [ ] Logging matches the required format
- [ ] Register preservation tests still pass
- [ ] Error injection tests still pass
- [ ] FD semantics unchanged unless documented
- [ ] VFS semantics unchanged unless documented
- [ ] `docs/syscalls.md` updated if ABI changed
- [ ] `docs/vfs.md` updated if file semantics changed
- [ ] `docs/dispatcher.md` updated if dispatch path changed

======================================================================

End of Document

