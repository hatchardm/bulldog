# Bulldog Kernel – ELF Loader (Planned User Program Loader)

This document describes Bulldog’s planned ELF loader: the component responsible for loading
user‑mode programs into memory, preparing their execution context, and transferring control
from the kernel to user space.

It is a forward‑looking design document, grounded in Bulldog’s existing architecture:

- Higher‑half kernel  
- Syscall interface  
- Privilege switching  
- Planned process model  
- Future VFS integration  

---

## 1. Overview

The ELF loader will:

- Parse ELF binaries (64‑bit, `x86_64`)  
- Map code and data segments into a process address space  
- Set up a user stack  
- Initialize a user‑mode context frame  
- Transfer control to user mode via `iretq`  
- Integrate with the process subsystem and scheduler  

It sits at the boundary between:

- **VFS / filesystem** (where the ELF file lives)  
- **Process subsystem** (where the program runs)  
- **Interface layer** (syscalls, privilege switching)  

---

## 2. ELF Format Support (Initial Scope)

Initial support will target:

- ELF64 (`EM_X86_64`)  
- Position‑dependent executables  
- Simple segment layout (no PIE at first)  
- Minimal dynamic linking (static binaries first)  

The loader will handle:

- ELF header  
- Program headers (`PT_LOAD`)  
- Entry point (`e_entry`)  

Sections (`.text`, `.data`, `.bss`, etc.) are not directly used for mapping — only segments.

---

## 3. Loading Pipeline

The ELF loading process will follow these steps:

1. **Open ELF file** via `open` (future VFS integration)  
2. **Read ELF header** and validate:  
   - Magic (`0x7F 'E' 'L' 'F'`)  
   - Class (64‑bit)  
   - Machine (`EM_X86_64`)  
3. **Read program headers**  
4. **Create a new process** structure  
5. **Allocate a page table** for the process  
6. **Map each `PT_LOAD` segment** into the process address space  
7. **Zero‑initialize `.bss` regions**  
8. **Set up user stack**  
9. **Initialize context frame**  
10. **Add process to scheduler**  
11. **Transfer control to user mode** (for the first process)

---

## 4. Segment Mapping

For each `PT_LOAD` program header:

- Compute virtual address range  
- Allocate physical pages  
- Map pages into the process page table  
- Copy file contents into mapped memory  
- Apply flags:  
  - Readable  
  - Writable  
  - Executable  

Example mapping:

- `.text` → RX  
- `.data` → RW  
- `.bss` → RW (zero‑filled)  

The loader must respect alignment and page boundaries.

---

## 5. User Stack Setup

The ELF loader will:

- Allocate a user stack region (e.g., top of user address space)  
- Map pages for the stack  
- Initialize stack pointer (`RSP`)  
- Optionally push:  
  - `argc`  
  - `argv`  
  - `envp`  
  - Auxiliary vector (future)  

Early versions may use a minimal stack with no arguments.

---

## 6. Initial Context Frame

To start execution, the loader builds a context frame compatible with `iretq`:

Frame contains:

- User RIP = ELF entry point (`e_entry`)  
- User CS (DPL=3)  
- User RFLAGS (interrupts enabled)  
- User RSP (top of user stack)  
- User SS (DPL=3)  

This frame is placed on the kernel stack, and the kernel executes:

```
iretq
```

to enter user mode.

This is the same mechanism used for syscall returns.

---

## 7. Process Integration

The ELF loader integrates with the process subsystem:

- Creates a `Process` structure  
- Sets `cr3` to the process page table  
- Stores initial context (RIP, RSP, flags)  
- Assigns a PID  
- Marks state as `running` or `ready`  
- Adds the process to the scheduler’s run queue  

Later, multiple processes will be supported.

See:  
`../architecture/processes.md`  
`process.md` (interface layer, future)

---

## 8. Syscall Interaction

Once a process is running, it interacts with the kernel via syscalls:

- `write` – output  
- `read` – input (future)  
- `open` – file access (future VFS)  
- `close` – FD management  
- `alloc` / `free` – memory management (future user allocator)  
- `exit` – process termination  

The ELF loader itself does not expose syscalls — it prepares the environment in which
syscalls are used.

See:  
`syscall.md`  
`syscall-table.md`  
`privilege-switching.md`

---

## 9. Error Handling

The ELF loader must handle:

- Invalid ELF magic  
- Unsupported machine type  
- Malformed program headers  
- Out‑of‑memory conditions  
- Mapping failures  

Errors will be reported via:

- Kernel logs  
- Syscall error codes (for user‑initiated exec, future)  

Early versions may simply panic on fatal loader errors.

---

## 10. Future Enhancements

Planned improvements:

- Support for PIE (position‑independent executables)  
- Dynamic linker / shared libraries  
- `execve` syscall for replacing process images  
- Argument and environment passing (`argc`, `argv`, `envp`)  
- Auxiliary vector (`auxv`)  
- Per‑process working directory and VFS integration  
- ELF loader test harness  

These will be documented as they are implemented.

---

## Contributor Notes

- Keep the loader minimal and deterministic  
- Validate all ELF headers before mapping  
- Avoid exposing loader internals to user mode  
- Document all new loader features here  
- Maintain compatibility with the process and syscall subsystems  

---

## License
MIT or Apache 2.0 — to be determined.
