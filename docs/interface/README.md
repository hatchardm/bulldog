# Bulldog Kernel – Interface Layer Overview

This directory contains documentation for Bulldog’s **interface layer** — the subsystem that
connects **user mode** to the **kernel core**.  
It sits *above* the architecture layer and defines how user processes interact with the
kernel through:

- Privilege switching  
- Syscalls  
- The syscall dispatcher  
- The syscall table  
- The syscall harness  
- The planned Virtual Filesystem (VFS)  

These documents describe the kernel’s external interface, not its internal architecture.

---

## Directory Purpose

The `interface/` directory provides:

- A stable ABI for user‑mode programs  
- A clear syscall model  
- A deterministic privilege‑switching path  
- A contributor‑friendly syscall table  
- A rigorous syscall harness for validation  
- A forward‑looking VFS design  

This layer is the **boundary** between user mode and kernel mode.

---

## Documents in This Directory

### 1. `privilege-switching.md`
Describes how Bulldog transitions between Ring 3 ↔ Ring 0:

- IDT gate permissions  
- TSS stack switching  
- Kernel entry/exit path  
- `iretq` return frame  
- Security guarantees  

This is the foundation of all syscalls.

---

### 2. `syscall.md`
Defines Bulldog’s syscall interface:

- ABI (register usage)  
- Syscall numbering (1–7)  
- Handler signatures  
- Kernel entry/exit behavior  
- Error handling  
- Contributor workflow  

This is the canonical reference for syscall behavior.

---

### 3. `syscall-table.md`
Documents the static syscall table:

- Current syscall numbers  
- Handler mapping  
- Table invariants  
- How to add new syscalls  
- Harness integration  

This is the single source of truth for syscall numbering.

---

### 4. `dispatcher.md`
Explains the syscall dispatcher:

- Lookup logic  
- Handler invocation  
- Error paths  
- Register preservation  
- Return‑value rules  

This is the core of syscall execution.

---

### 5. `syscall-harness-guide.md`
Describes the syscall test harness:

- Register preservation tests  
- Privilege‑switching tests  
- Table validation  
- Error‑path validation  
- Deterministic logging  

This ensures syscall correctness before merging.

---

### 6. `vfs.md`
Defines Bulldog’s planned Virtual Filesystem:

- VFS nodes  
- VFS operations  
- File descriptor tables  
- Path resolution  
- Mount table  
- Driver model  

This is a forward‑looking design document for future filesystem work.

---

## Relationship to the Architecture Layer

The interface layer depends on the architecture layer:

- GDT/TSS → privilege switching  
- IDT → syscall entry vector  
- Kernel stacks → syscall entry safety  
- Memory subsystem → user pointer validation  
- APIC timer → future scheduling  

But it is **not** part of the architecture itself.

Architecture = hardware‑facing  
Interface = user‑facing

Keeping these layers separate makes Bulldog easier to understand and contribute to.

---

## Contributor Workflow

When adding or modifying interface‑layer features:

1. Update the relevant document in this directory  
2. Update the syscall table if adding a syscall  
3. Update the syscall harness tests  
4. Validate privilege switching if needed  
5. Keep ABI changes backward‑compatible  
6. Maintain deterministic behavior  

All interface changes must be documented here.

---

## Future Expansion

Planned additions to this directory:

- `elf-loader.md` (user program loading)  
- `process.md` (user‑mode process model)  
- `scheduler.md` (preemptive scheduling)  
- `signals.md` (user‑mode signal delivery)  
- `ipc.md` (inter‑process communication)  

These will be added as Bulldog’s user‑mode subsystem grows.

---

## License

MIT or Apache 2.0 — TBD.
