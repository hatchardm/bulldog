# Bulldog Kernel – Architecture Layer Index

The architecture layer contains documentation for all **hardware‑facing subsystems** in the
Bulldog kernel. These documents describe how the kernel interacts with the CPU, memory
hardware, interrupt controllers, and low‑level system structures.

This layer is distinct from the interface layer, which describes user‑mode interaction and
syscalls.

---

## Subsystems

### Memory & Paging
- `memory.md`  
- `layout.md`  
- `kernel_init.md` (memory map integration)

### GDT / TSS
- `gdt.md`  
- `stack.md` (kernel stack model)

### Interrupts & Exceptions
- `interrupts.md`  
- `time.md` (APIC timer integration)

### APIC / LAPIC
- `apic.md`  
- `smp.md` (multi‑CPU initialization)

### Kernel Layout & Linker
- `layout.md`  
- `linker.md`

### Boot & Initialization
Boot documentation is now located in:

```
../boot/boot.md
```

---

## Purpose of This Layer

The architecture layer:

- Defines CPU‑level invariants  
- Documents hardware‑facing subsystems  
- Provides contributor guidance for low‑level features  
- Ensures architectural clarity and reproducibility  
- Serves as the foundation for the interface layer

All architecture changes must be documented here.

---

## Related Layers

### Interface Layer
User‑mode boundary, syscalls, VFS, ELF loader:

```
../interface/index.md
```

### Boot Layer
Early kernel initialization:

```
../boot/README.md
```

---

## Contributor Notes

- Keep architecture documents technically precise  
- Document unsafe blocks with justification  
- Maintain subsystem boundaries  
- Update this index when adding new architecture documents

---

## License
MIT or Apache 2.0 — to be determined.
