# Bulldog Kernel – Interface Layer Index

The interface layer contains documentation for all **user‑visible kernel interfaces**.  
It defines how user mode interacts with the kernel through syscalls, privilege switching,
process management, and future subsystems such as VFS and ELF loading.

This layer sits above the architecture layer.

---

## Subsystems

### Privilege Switching
- `privilege-switching.md`  
Describes Ring 3 ↔ Ring 0 transitions, TSS stack switching, and `iretq` return path.

### Syscall Interface
- `syscall.md`  
- `syscall-table.md`  
- `dispatcher.md`  
- `syscall-harness-guide.md`  

Defines the syscall ABI, numbering, dispatcher, and validation harness.

### Virtual Filesystem (Planned)
- `vfs.md`  
Design for FD tables, VFS nodes, drivers, and mount table.

### ELF Loader (Planned)
- `elf-loader.md`  
User program loading, segment mapping, context frame creation.

### Process Model
- `process.md`  
User‑mode process lifecycle, FD table, scheduling interface, termination.

---

## Purpose of This Layer

The interface layer:

- Defines the kernel’s external API  
- Documents user‑mode execution  
- Ensures syscall ABI stability  
- Provides contributor guidance for interface‑level features  
- Serves as the boundary between user mode and kernel mode

All interface changes must be documented here.

---

## Related Layers

### Architecture Layer
Hardware‑facing subsystems:

```
../architecture/index.md
```

### Boot Layer
Early kernel initialization:

```
../boot/README.md
```

---

## Contributor Notes

- Maintain ABI stability  
- Keep documentation synchronized with implementation  
- Ensure syscall harness tests remain deterministic  
- Update this index when adding new interface documents

---

## License
MIT or Apache 2.0 — to be determined.
