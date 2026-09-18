# Bulldog Kernel – Boot & Initialization Subsystem

This directory contains documentation for Bulldog’s **boot and early initialization**
subsystem. It covers everything that happens before the kernel enters its main architecture
and interface layers.

Boot is a standalone subsystem because it spans:

- Firmware / bootloader interaction  
- Early memory map setup  
- Higher‑half kernel relocation  
- Early logging  
- APIC initialization  
- Transition into full kernel initialization  

---

## Documents

### Boot Process
- `boot.md`  
Describes the early boot sequence, memory map, stack setup, HHDM mapping, and transition to
kernel initialization.

More documents will be added as the boot subsystem expands.

---

## Purpose of This Layer

The boot layer:

- Defines the earliest kernel invariants  
- Documents memory map and relocation  
- Establishes initial CPU state  
- Prepares architecture subsystems  
- Bridges firmware → kernel execution  

Boot is intentionally isolated from the architecture layer to keep subsystem boundaries
clean and contributor‑friendly.

---

## Related Layers

### Architecture Layer
Hardware‑facing subsystems:

```
../architecture/index.md
```

### Interface Layer
User‑mode boundary and syscalls:

```
../interface/index.md
```

---

## Contributor Notes

- Keep boot documentation precise and minimal  
- Avoid mixing boot logic with architecture or interface layers  
- Document all early‑boot assumptions and invariants  
- Update this index when adding new boot documents

---

## License
MIT or Apache 2.0 — to be determined.
