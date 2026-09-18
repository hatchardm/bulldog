# Bulldog Kernel – `execve` Syscall (Planned)

This document describes the planned `execve` syscall, which replaces the current process image
with a new ELF program and provides arguments and environment variables.

It complements:

- `exec.md`
- `process.md`
- `elf-loader.md`
- `scheduler.md`

---

## 1. Purpose

`execve` loads a new program into the current process:

- Replaces address space  
- Replaces registers  
- Replaces stack  
- Replaces file descriptors (optional)  
- Starts execution at ELF entry point  

Unlike `fork`, `execve` does not create a new process.

---

## 2. Proposed ABI

```
execve(path: *const u8, argv: *const *const u8, envp: *const *const u8)
    -> SysResult
```

Arguments:

- `path` – null‑terminated program path  
- `argv` – argument vector  
- `envp` – environment vector  

Return value:

- Does not return on success  
- Returns error code on failure  

---

## 3. Kernel Responsibilities

When `execve` is invoked:

1. Validate pointers  
2. Resolve path via VFS  
3. Load ELF file  
4. Destroy current address space  
5. Create new page table  
6. Map ELF segments  
7. Build new user stack containing `argv` and `envp`  
8. Transfer control via `iretq`  

---

## 4. Interaction With VFS

`execve` depends on:

- `open`  
- `read`  
- Path resolution  
- VFS nodes  
- Mount table  

---

## 5. Error Handling

Possible errors:

- File not found  
- Invalid ELF  
- Out of memory  
- Mapping failure  
- Permission denied (future)  

---

## 6. Future Enhancements

- Auxiliary vector (`auxv`)  
- Dynamic linker support  
- Interpreter scripts (`#!`)  
- Security model (permissions, capabilities)  

---

## License
MIT or Apache 2.0 — to be determined.
