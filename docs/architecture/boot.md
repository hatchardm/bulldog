# Bulldog Kernel – Boot and Higher‑Half Initialization

This document describes Bulldog’s boot process, including early identity mapping, transition
to the higher‑half kernel, stack initialization, and early subsystem setup. It provides
contributors with a clear understanding of how control flows from the bootloader into the
Bulldog kernel on the `x86_64-bulldog` architecture.

Bulldog relies on a multistage boot process that prepares paging, memory maps, and CPU state
before transferring control to the higher‑half kernel entry point.

---

## Boot Overview

The Bulldog boot sequence follows these stages:

1. Bootloader loads the kernel ELF  
2. Bootloader provides memory map and CPU state  
3. Early identity‑mapped paging is installed  
4. Higher‑half kernel mapping is activated  
5. Execution jumps to the higher‑half entry point  
6. Kernel initializes core subsystems  
7. Interrupts and APIC are configured  
8. Kernel enters main initialization routine  

This sequence ensures a deterministic and debuggable startup path.

---

## Bootloader Responsibilities

Bulldog expects the bootloader to provide:

- A valid ELF64 kernel image loaded into memory  
- A memory map describing usable and reserved regions  
- Initial CPU state in long mode  
- Identity‑mapped paging for low memory  
- A stack for early execution  
- A pointer to boot information structures  

Bulldog does not rely on BIOS or legacy real‑mode services.

---

## Early Paging

Before the kernel can run in the higher half, paging must be active.  
The bootloader installs:

- Identity mapping for low memory (0x0000_0000_0000_0000 → 0x0000_0000_0000_0000)  
- Temporary mapping for the kernel’s physical load address  

This allows the kernel to execute early initialization code before switching to its final
virtual address layout.

---

## Higher‑Half Kernel Mapping

Bulldog is linked to run at a higher‑half virtual address, typically above:

```
0xFFFF_8000_0000_0000
```

To transition into the higher half:

1. Kernel page tables map the kernel’s physical load region into the higher‑half region  
2. CR3 is updated to point to the kernel’s page tables  
3. A far jump transfers execution to the higher‑half entry point  

Diagram:

```
Physical Load Address (e.g., 0x0010_0000)
        ↓
Mapped to Higher Half (e.g., 0xFFFF_8000_0010_0000)
        ↓
Execution continues in higher‑half virtual space
```

This ensures:

- Kernel addresses are stable  
- User space cannot access kernel memory  
- Direct physical mapping can be established  

---

## Kernel Entry Point

The higher‑half entry point performs:

- Stack initialization  
- Zeroing of BSS  
- Relocation fixups (if needed)  
- Initialization of logging and early console  
- Setup of the direct physical map  
- Initialization of the physical memory allocator  

Once these steps are complete, the kernel can safely initialize interrupts and APIC.

---

## Early Stack Setup

The bootloader provides an initial stack, but Bulldog replaces it with a kernel‑owned stack
as soon as paging is stable.

Stack invariants:

- 16‑byte alignment  
- Guard page below the stack (future)  
- One stack per CPU (future SMP support)  

---

## Memory Map Processing

The bootloader provides a memory map describing:

- Usable RAM  
- Reserved regions  
- ACPI tables  
- MMIO regions  
- Bootloader structures  

Bulldog parses this map to populate the early physical frame allocator.

---

## Transition to Kernel Main

Once paging, memory, and stacks are initialized, control transfers to the main kernel
initialization routine.

This stage initializes:

- GDT/TSS  
- IDT  
- APIC and LAPIC timer  
- Syscall vector (0x80)  
- Kernel heap  
- Logging subsystem  
- Future: scheduler and process model  

Diagram:

```
bootloader
    ↓
early kernel (identity map)
    ↓
higher‑half entry
    ↓
paging + memory + stacks
    ↓
interrupts + APIC
    ↓
kernel main
```

---

## Future Boot Enhancements

Planned improvements:

- SMP boot and per‑CPU initialization  
- ACPI parsing  
- Early framebuffer console  
- Kernel command‑line arguments  
- Boot‑time logging buffer  
- KASLR (kernel address space layout randomization)  

---

## Contributor Notes

- Keep boot diagrams updated as the kernel evolves  
- Document all new mappings and address transitions  
- Ensure early logging remains deterministic  
- Avoid complex logic before higher‑half transition  
- Validate stack alignment and guard pages  

---

## License

MIT or Apache 2.0 — to be determined.
