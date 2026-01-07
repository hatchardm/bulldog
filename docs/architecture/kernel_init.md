# Bulldog Kernel – Kernel Initialization (`kernel_init` in `lib.rs`)

This document describes Bulldog’s *current* kernel initialization sequence as implemented in the `kernel_init` function located in `src/lib.rs`. It reflects the real behavior of:

- `memory.rs` (paging, allocators, LAPIC MMIO mapping)
- `gdt.rs` and `stack.rs` (GDT, TSS, IST stacks)
- `interrupts` (IDT and LAPIC timer vector)
- `syscall` subsystem (syscall entry setup)
- `allocator` (kernel heap)
- `apic.rs` (LAPIC initialization)

This is a description of the *current implementation*, not future user‑mode or SMP plans.

---

# 1. Purpose of `kernel_init`

`kernel_init` is the central bootstrap routine executed after the bootloader hands control to the kernel. It is responsible for:

- Establishing paging and memory allocators  
- Initializing the kernel heap  
- Setting up CPU descriptor tables (GDT, IDT, TSS)  
- Registering the syscall entry point  
- Mapping and initializing the LAPIC  
- Preparing the LAPIC IST stack  
- Enabling interrupts  

Once `kernel_init` completes, the kernel is fully operational and ready to handle interrupts, syscalls, and memory allocation.

---

# 2. High‑Level Initialization Order

The real initialization sequence is:

1. Disable legacy PIC  
2. Create the paging mapper  
3. Log physical memory regions  
4. Create the temporary pre‑heap frame allocator  
5. Initialize the kernel heap  
6. Initialize the file descriptor table  
7. Finalize the full frame allocator  
8. Log memory regions with virtual addresses  
9. Initialize GDT and IDT  
10. Register syscall handler  
11. (Optional) run syscall tests  
12. Map LAPIC MMIO  
13. Map and mark LAPIC IST stack pages  
14. Mark used frames in the bitmap allocator  
15. Remap LAPIC stack pages with correct flags  
16. Initialize LAPIC (timer, SVR, ID/version)  
17. Enable interrupts  

This sequence reflects the exact order in the source code.

---

# 3. Paging and Mapper Initialization

The kernel begins by creating the paging mapper:

```rust
let mut mapper = unsafe { init_offset_page_table(phys_mem_offset) };
```

This uses the bootloader‑provided physical memory offset to access page tables through the direct map.

Memory regions from the bootloader are logged for debugging.

---

# 4. Temporary Pre‑Heap Frame Allocator

Before the heap exists, Bulldog uses a temporary allocator:

```rust
let (temp_frames, memory_map) = BootInfoFrameAllocator::init_temp(memory_regions);
let mut temp_allocator = PreHeapAllocator { ... };
```

This allocator:

- Provides up to 512 frames  
- Is used only during heap initialization  
- Is deterministic and simple  

---

# 5. Kernel Heap Initialization

The heap is initialized using the temporary allocator:

```rust
allocator::init_heap(&mut mapper, &mut temp_allocator)
```

Once the heap is ready, higher‑level structures (like `Box`, `Vec`, `BTreeMap`) become usable.

The FD table is initialized immediately afterward:

```rust
init_fd_table_with_std();
```

---

# 6. Final Frame Allocator

After the heap is available, the kernel constructs the full bitmap‑based frame allocator:

```rust
let frames = temp_allocator.into_vec();
let mut frame_allocator = BootInfoFrameAllocator::new(memory_map, frames);
```

This allocator:

- Tracks frames using a bitmap  
- Supports marking used frames  
- Will eventually support deallocation (future work)  

---

# 7. Logging Virtual Memory Regions

The kernel logs each memory region again, this time with its virtual address computed via:

```
virt = region.start + phys_mem_offset
```

This helps verify the direct physical map.

---

# 8. CPU Tables: GDT and IDT

The kernel initializes:

- **GDT** (code/data segments + TSS)
- **IDT** (interrupt handlers)

```rust
gdt::init();
interrupts::init_idt();
```

This must occur before enabling interrupts or LAPIC timers.

---

# 9. Syscall Initialization

The syscall entry point is registered:

```rust
crate::syscall::init_syscall();
```

This must occur **before interrupts are enabled**, ensuring the syscall vector is valid.

If the `syscall_tests` feature is enabled, the syscall harness runs here.

---

# 10. LAPIC MMIO Mapping

The LAPIC MMIO region is mapped into higher‑half kernel space:

```rust
map_lapic_mmio(&mut mapper, &mut frame_allocator);
```

This maps:

- Virtual: `LAPIC_VIRT_BASE`
- Physical: `0xFEE0_0000`

The LAPIC cannot be accessed until this mapping exists.

---

# 11. LAPIC IST Stack Mapping

The LAPIC timer and page‑fault handlers run on IST1, backed by `LAPIC_STACK`.

`kernel_init`:

1. Computes the virtual range of the LAPIC stack  
2. Marks all frames backing the stack as “used” in the bitmap allocator  
3. Remaps each page with `PRESENT | WRITABLE` flags  
4. Flushes TLB entries after each unmap/map  

This ensures:

- The LAPIC IST stack is fully mapped  
- No pages are accidentally reused  
- The stack has correct permissions  

This is a unique part of Bulldog’s initialization pipeline.

---

# 12. Marking Used Frames

The frame allocator marks all non‑usable frames:

```rust
frame_allocator.mark_used_frames();
```

Then logs each used frame for debugging.

---

# 13. LAPIC Initialization

The LAPIC is initialized:

```rust
setup_apic();
```

This performs:

- MSR read of LAPIC base  
- Enable bit verification  
- Logging LAPIC ID and version  
- Writing SVR (enable + spurious vector)  
- Configuring periodic timer  
- Setting initial count  
- Logging current count  

---

# 14. Enabling Interrupts

Finally, interrupts are enabled:

```rust
x86_64::instructions::interrupts::enable();
```

At this point:

- GDT/TSS are active  
- IDT is active  
- Syscalls are registered  
- LAPIC timer is running  
- Heap is initialized  
- Frame allocator is active  

The kernel is fully operational.

---

# 15. Future Work

Planned improvements:

- Guard pages around IST stacks  
- User‑mode address spaces  
- Per‑process page tables  
- IOAPIC initialization  
- SMP bring‑up and IPIs  
- Scheduler integration with LAPIC timer  
- Dynamic LAPIC timer calibration  

---

# 16. Contributor Notes

- `kernel_init` must run before any interrupts occur  
- LAPIC MMIO mapping must precede `setup_apic()`  
- Syscall handler must be registered before enabling interrupts  
- IST stacks must remain aligned and unmoved  
- Logging is essential for early bring‑up debugging  

---

# License

MIT or Apache 2.0 — TBD.