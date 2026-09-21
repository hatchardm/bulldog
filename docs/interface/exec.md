# Bulldog Kernel – `exec` Syscall (Planned)

This document describes the planned `exec` syscall, which replaces the current process image
with a new ELF program. It is part of Bulldog’s future process‑creation and user‑mode
execution model.

It complements:

- `process.md`
- `elf-loader.md`
- `syscall.md`
- `syscall-table.md`

---

## 1. Purpose

`exec` allows a running process to:

- Replace its address space  
- Load a new ELF binary  
- Reset its user stack  
- Reset its instruction pointer  
- Begin executing the new program  

Unlike `fork`, `exec` does not create a new process.

---

## 2. Proposed ABI

```
exec(path: *const u8) -> SysResult
```

Arguments:

- `path` – null‑terminated string identifying the ELF file

Return value:

- On success: does not return (new program begins)  
- On failure: returns an error code

---

## 3. Kernel Responsibilities

When `exec` is invoked:

1. Validate user pointer  
2. Resolve path via VFS  
3. Load ELF file  
4. Destroy current address space  
5. Create new page table  
6. Map ELF segments  
7. Allocate new user stack  
8. Build new context frame  
9. Transfer control via `iretq`

The process ID remains the same.

---

## 4. Interaction With VFS

`exec` depends on:

- `open`  
- `read`  
- Path resolution  
- VFS nodes  
- Mount table  

The ELF loader reads the file through VFS.

---

## 5. Error Handling

Possible errors:

- File not found  
- Invalid ELF  
- Out of memory  
- Mapping failure  
- Permission denied (future)  

Errors return via the syscall ABI.

---

## 6. Future Enhancements

- `execve` with `argv` and `envp`  
- Auxiliary vector (`auxv`)  
- Interpreter / dynamic linker support  
- Security model (permissions, capabilities)

---

## Contributor Notes

- Keep `exec` semantics consistent with POSIX  
- Ensure deterministic address‑space replacement  
- Document all new error codes  
- Update syscall table when adding `exec`

---

## License
MIT or Apache 2.0 — to be determined.
