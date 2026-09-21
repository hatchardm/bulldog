# Bulldog Kernel – Bootloader Handoff (UEFI / OVMF)

This document describes how Bulldog receives control from a UEFI bootloader (typically
OVMF), how boot information is passed to the kernel, and how the kernel transitions from
firmware context into its own higher‑half execution environment.

This subsystem defines the earliest invariants Bulldog relies on before entering full kernel
initialization.

---

## 1. Execution Environment

Bulldog is loaded by a UEFI bootloader in **long mode**, with:

- Paging enabled  
- Identity mapping for low memory  
- Kernel ELF image loaded into memory  
- CPU in 64‑bit mode  
- Interrupts disabled  
- A valid stack  
- Boot information structures provided  

The bootloader is responsible for preparing a minimal environment that allows the kernel to
perform its own initialization.

---

## 2. Boot Information Structure

The bootloader provides a `BootInfo` structure containing:

- Physical memory map  
- Kernel physical load address  
- Kernel entry point  
- Framebuffer information (future)  
- ACPI tables (future)  
- Bootloader metadata  

The kernel uses this structure to initialize:

- Higher‑half direct map (HHDM)  
- Physical memory allocator  
- Early logging  
- Framebuffer console (future)

---

## 3. Memory Map Retrieval

The bootloader retrieves the UEFI memory map via:

```
GetMemoryMap()
```

and passes it to the kernel as a slice of memory descriptors.

Each descriptor contains:

- Physical base  
- Length  
- Type (usable, reserved, ACPI, MMIO, etc.)  
- Attributes  

The kernel converts these descriptors into its own `MemoryRegion` representation.

---

## 4. ExitBootServices

Before the kernel can safely manage memory, the bootloader must call:

```
ExitBootServices(image_handle, map_key)
```

This:

- Terminates all UEFI runtime services  
- Locks the memory map  
- Prevents further firmware allocation  
- Hands full control to the kernel  

The kernel must not call UEFI functions after this point.

---

## 5. Kernel Entry Point

After `ExitBootServices`, the bootloader jumps to the kernel’s entry point:

```
kernel_entry(&BootInfo)
```

The kernel entry point performs:

- Early stack setup  
- BSS zeroing  
- Higher‑half paging activation  
- Direct physical map setup  
- Logging initialization  
- Physical memory allocator initialization  
- Transition to `kernel_main`

---

## 6. Higher‑Half Transition

Bulldog is linked to run at:

```
0xFFFF_8000_0000_0000
```

The bootloader provides temporary identity mapping.  
The kernel installs its own page tables and performs a far jump into the higher half.

This ensures:

- Kernel addresses are stable  
- User space cannot access kernel memory  
- Direct physical mapping is available

---

## 7. Future Bootloader Features

Planned enhancements:

- Framebuffer console initialization  
- ACPI table handoff  
- Kernel command‑line arguments  
- KASLR (address randomization)  
- SMP boot coordination  
- Boot‑time logging buffer  

---

## Contributor Notes

- Keep bootloader assumptions minimal  
- Document all early‑boot invariants  
- Avoid mixing boot logic with architecture or interface layers  
- Update this document when adding new boot‑time features

---

## License
MIT or Apache 2.0 — to be determined.
