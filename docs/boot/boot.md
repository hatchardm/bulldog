# Bulldog Kernel – Boot and Higher‑Half Initialization (Updated 2026)

This document describes Bulldog’s **current, real** boot process as implemented in the new Bulldog bootloader and kernel. It replaces older Linux‑dev era notes and reflects the actual behavior of:

- `bootloader/src/main.rs`
- `kernel/src/lib.rs` (`kernel_main` + `kernel_init`)
- `memory.rs` (paging + direct map + LAPIC MMIO)
- `gdt.rs` + `stack.rs` (GDT, TSS, IST stacks)
- `interrupts.rs` (IDT + LAPIC timer vector)
- `apic.rs` (LAPIC bring‑up)

This is a description of the **current implementation**, not future SMP or user‑mode plans.

---

## 1. Boot Overview

Bulldog’s boot sequence consists of:

1. Bootloader loads kernel ELF into physical memory  
2. Bootloader initializes GOP framebuffer  
3. Bootloader constructs BootInfo (framebuffer, memory map, physical offset)  
4. Bootloader installs identity paging  
5. Bootloader jumps to kernel entry  
6. Kernel switches to higher‑half mapping  
7. Kernel initializes memory + heap + frame allocator  
8. Kernel initializes GDT/TSS + IDT  
9. Kernel maps LAPIC MMIO + configures APIC  
10. Kernel enables interrupts + LAPIC timer  
11. Kernel enters `hlt_loop()`

This sequence is deterministic and fully logged.

---

## 2. Bootloader Responsibilities (Current Implementation)

### 2.1 Kernel ELF loading
- Loads the kernel ELF64 image into physical memory  
- Ensures `.text`, `.rodata`, `.data`, `.bss` are placed correctly  
- Logs ELF load address and section boundaries

### 2.2 Framebuffer initialization
Bootloader initializes GOP and provides:

- physical framebuffer address  
- virtual framebuffer address  
- width, height, stride  
- pixel format  

Kernel uses this for early framebuffer console.

### 2.3 Memory map handoff
Bootloader passes a complete memory map:

- Usable RAM  
- Reserved regions  
- ACPI tables  
- MMIO regions  
- Bootloader structures  

Kernel consumes this to build the frame allocator.

### 2.4 Physical memory offset (HHDM)
Bootloader provides:

```
boot_info.physical_memory_offset = 0xFFFF_8000_0000_0000
```

This creates the **direct physical map**:

```
virt = phys + phys_mem_offset
```

Used for:

- page‑table access  
- frame allocator  
- LAPIC stack mapping  
- memory map logging

### 2.5 Identity paging
Bootloader installs:

- identity map for low memory  
- temporary mapping for kernel load region  
- mapping for framebuffer  
- mapping for BootInfo  

This allows the kernel to run before switching to higher‑half.

### 2.6 Jump to kernel
Bootloader jumps to:

```
kernel_entry(boot_info_ptr)
```

with:

- long mode active  
- paging active  
- stack valid  
- interrupts disabled

---

## 3. Kernel Entry (`kernel_main`)

The kernel entry point performs:

1. Framebuffer setup  
2. Writer initialization  
3. Logger initialization  
4. Memory map extraction  
5. Call into `kernel_init()`  
6. Enter `hlt_loop()`

All heavy lifting happens inside `kernel_init`.

---

## 4. Higher‑Half Paging

Bulldog is linked to run at:

```
0xFFFF_8000_0000_0000
```

Kernel constructs an `OffsetPageTable`:

```rust
let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
let mut mapper = unsafe { init_offset_page_table(phys_mem_offset) };
```

This enables:

- direct physical memory access  
- page‑table manipulation  
- higher‑half virtual addressing  

No CR3 switch is needed — bootloader paging already supports higher‑half execution.

---

## 5. Memory Map Processing

Kernel logs each memory region:

```
kind=Usable, start=0x..., end=0x...
kind=Reserved, start=0x..., end=0x...
kind=Acpi, ...
kind=Mmio, ...
```

Then constructs:

- temporary pre‑heap allocator  
- full bitmap frame allocator  

Allocator is used for:

- heap initialization  
- LAPIC stack mapping  
- future page allocations

---

## 6. Kernel Heap Initialization

Heap is initialized via:

```rust
allocator::init_heap(&mut mapper, &mut temp_allocator);
```

Once heap is online:

- `Box`, `Vec`, `BTreeMap` become usable  
- FD table is initialized (`init_fd_table_with_std()`)  
- VFS becomes usable  
- Syscalls can allocate memory

---

## 7. GDT, TSS, and IST Stacks

Kernel initializes:

- kernel code segment  
- kernel data segment  
- TSS with two IST stacks:
  - IST0 → double fault  
  - IST1 → LAPIC timer + page fault  

Stacks are:

- 128 KiB  
- statically allocated  
- 16‑byte aligned  
- mapped in higher‑half space

Must occur **before enabling interrupts**.

---

## 8. IDT Initialization

`interrupts::init_idt()` installs:

- all CPU exception handlers  
- IST‑backed page fault + double fault  
- LAPIC timer vector  
- spurious vector  
- default handlers for unused vectors  
- skips syscall vector (owned by syscall subsystem)

IDT is loaded via `lidt`.

---

## 9. Syscall Vector Initialization

Syscall subsystem installs vector `0x80`:

```rust
init_syscall();
```

This sets:

- DPL=3  
- handler = `syscall_handler`  
- ABI = `rax, rdi, rsi, rdx`

Syscalls are now ready for usermode (future).

---

## 10. LAPIC MMIO Mapping

Kernel maps LAPIC MMIO:

```
virt: 0xFFFF_FF00_0000_0000
phys: 0xFEE0_0000
flags: PRESENT | WRITABLE | NO_EXECUTE
```

Must occur **before** calling `setup_apic()`.

---

## 11. LAPIC IST Stack Mapping

Kernel:

1. Computes LAPIC stack virtual range  
2. Marks frames as used  
3. Remaps pages with correct flags  
4. Flushes TLB entries  

Ensures LAPIC timer + page faults run on a safe stack.

---

## 12. LAPIC Initialization

`setup_apic()` performs:

- MSR read of LAPIC base  
- enable bit check  
- ID + version logging  
- SVR write (enable + spurious vector)  
- timer divisor = 16  
- LVT_TIMER = periodic mode  
- INITIAL_COUNT = 500,000  
- CURRENT_COUNT logging  

After this, LAPIC timer interrupts begin firing.

---

## 13. Interrupts Enabled

Finally:

```rust
x86_64::instructions::interrupts::enable();
```

At this point:

- GDT/TSS active  
- IDT active  
- LAPIC timer running  
- heap initialized  
- frame allocator active  
- syscalls registered  

Kernel enters:

```rust
hlt_loop();
```

---

## 14. Future Boot Enhancements

Planned:

- SMP bring‑up  
- IOAPIC initialization  
- ACPI parsing  
- guard pages  
- usermode entry  
- scheduler  
- KASLR

---

## 15. Contributor Notes

- Bootloader must provide correct physical offset  
- LAPIC MMIO must be mapped before APIC setup  
- GDT/TSS must be initialized before interrupts  
- IDT must be loaded before enabling interrupts  
- Syscall vector must be installed before usermode  
- IST stacks must remain aligned and unmoved

---

## License

MIT or Apache 2.0 — TBD.
