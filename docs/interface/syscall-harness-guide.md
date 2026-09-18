# Bulldog Kernel – Syscall Harness Guide (Current Implementation)

This document describes Bulldog’s *current* syscall harness: the testing framework used to
validate syscall entry, dispatch, register preservation, privilege switching, and return‑path
correctness.

It reflects the real behavior of:

- `syscall.rs` (entry + dispatcher)
- `syscall_table.rs` (mapping)
- `privilege-switching.md` (Ring 3 ↔ Ring 0)
- `dispatcher.md` (handler dispatch)
- The user‑mode test program
- The kernel‑side logging pipeline

This is not a future design document — it describes exactly how Bulldog’s syscall harness
works today.

---

## 1. Purpose of the Syscall Harness

The syscall harness ensures:

- Syscalls enter the kernel correctly  
- Syscalls dispatch to the correct handler  
- Registers follow the syscall ABI  
- Kernel stack switching works  
- The return path via `iretq` is correct  
- User mode resumes execution safely  
- Errors are reported deterministically  

It is the single most important tool for validating privilege switching.

---

## 2. Harness Architecture

The harness consists of:

### 1. A user‑mode test program  
Runs in Ring 3 and issues syscalls with controlled register values.

### 2. Kernel logging  
Captures:

- Entry register state  
- Syscall number  
- Arguments  
- Handler return value  
- Exit register state  
- Return‑path correctness  

### 3. Validation logic  
Checks:

- Register preservation  
- Correct return value  
- Correct privilege level  
- Correct stack switching  
- Correct error handling  

### 4. Structured output  
Printed to serial/log for analysis.

---

## 3. User‑Mode Test Program

The test program:

- Sets registers to known values  
- Issues syscalls using `int SYSCALL_VECTOR`  
- Captures return values  
- Logs results  
- Repeats tests for all syscalls in the table  

Example pattern:

```
rax = syscall_number
rdi = arg0
rsi = arg1
rdx = arg2
r10 = arg3
r8  = arg4
r9  = arg5

int SYSCALL_VECTOR
```

The program verifies:

- Correct return value  
- No corruption of non‑argument registers  
- Correct privilege return  

---

## 4. Kernel‑Side Logging

The kernel logs:

- Syscall number  
- Arguments  
- Entry register state  
- Handler return value  
- Exit register state  
- Privilege level  
- Stack pointer before/after  
- Any errors  

Example log structure:

```
SYSCALL ENTRY:
  num = 3
  args = (1, 2, 3)
  rip = 0x400123
  rsp = 0x7FFFFFFFD000
  cs  = 0x23

DISPATCH:
  handler = sys_write
  result  = 3

RETURN:
  rip = 0x40012A
  rsp = 0x7FFFFFFFD000
  cs  = 0x23
```

This output is deterministic and easy to diff.

---

## 5. Register Preservation Tests

The harness verifies that:

- Only `rax` changes (return value)
- All other registers remain unchanged
- Flags remain unchanged
- Segment selectors remain unchanged

This is one of Bulldog’s strongest invariants.

If any register is corrupted, the harness prints:

```
REGISTER MISMATCH: rbx expected 0x1234, got 0x5678
```

---

## 6. Error Path Tests

The harness tests:

### Invalid syscall number  
Expect:

- `SysError::InvalidSyscall`
- `rax = error_code`
- No register corruption

### Bad arguments  
Expect:

- `SysError::BadAddress`
- No kernel crash
- No privilege leak

### Unimplemented syscalls  
Expect:

- `SysError::NotImplemented`

All error paths must be deterministic.

---

## 7. Privilege Switching Tests

The harness validates:

- Kernel entry uses TSS.rsp0  
- Kernel exit uses user RSP  
- `iretq` restores Ring 3  
- CS = user code segment  
- SS = user stack segment  

If privilege switching fails, the harness prints:

```
PRIVILEGE ERROR: returned to ring 0
```

This is fatal.

---

## 8. Stack Switching Tests

The harness checks:

- Kernel stack pointer changes on entry  
- Kernel stack pointer is restored on exit  
- User stack pointer is unchanged  
- No corruption of kernel stack frames  

If stack switching fails:

```
STACK ERROR: kernel stack corrupted
```

---

## 9. Dispatcher Tests

The harness verifies:

- Correct handler lookup  
- Correct argument passing  
- Correct return value  
- No mutation of non‑argument registers  
- No mutation of the syscall table  

If dispatch fails:

```
DISPATCH ERROR: wrong handler for syscall 4
```

---

## 10. Syscall Table Tests

The harness checks:

- Table ordering  
- Table completeness  
- No duplicate syscall numbers  
- No missing handlers  
- No runtime mutation  

If the table is incorrect:

```
TABLE ERROR: syscall 7 missing handler
```

---

## 11. Adding New Syscalls

When adding a new syscall:

1. Add handler to `syscall_table.rs`  
2. Document syscall in `syscall.md`  
3. Add harness tests  
4. Validate register preservation  
5. Validate error paths  
6. Validate privilege switching  

The harness must pass before merging.

---

## 12. Contributor Notes

- The harness is mandatory for all syscall development  
- All new syscalls must be tested  
- All register preservation rules must be followed  
- All privilege switching rules must be followed  
- All return‑path rules must be followed  
- Keep harness output deterministic  

---

## License

MIT or Apache 2.0 — TBD.

