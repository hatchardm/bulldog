## Bulldog Kernel – Page Table Structures

Reviewed: November 2026

This document describes the x86_64 page-table structures used by Bulldog.

It complements:

    paging.md
    hhdm.md
    memory.md
    frame-allocator.md

This document focuses on page-table organization, flags, and usage rather than paging initialization.

---

## 1. Page Table Hierarchy

Bulldog uses the standard x86_64 four-level page-table hierarchy.

Current levels:

    PML4
      ↓
    PDPT
      ↓
    PD
      ↓
    PT
      ↓
    4 KiB Pages

The hierarchy is inherited from the bootloader and later managed by the kernel.

---

## 2. PML4 (Level 4)

The Page Map Level 4 table is the root of the paging hierarchy.

Responsibilities:

    Select PDPT entries
    Separate kernel and user regions
    Provide access to all virtual memory

Current layout:

    Entries < 256
        Lower-half virtual space

    Entries >= 256
        Higher-half kernel space

Bulldog uses higher-half addresses for all kernel execution.

---

## 3. PDPT (Level 3)

The Page Directory Pointer Table selects Page Directories.

Current responsibilities:

    Intermediate address translation
    HHDM support
    Kernel-space mappings

The architecture supports:

    1 GiB huge pages

but Bulldog does not currently create them.

---

## 4. PD (Level 2)

The Page Directory selects Page Tables.

Responsibilities:

    Intermediate translation
    Mapping organization

The architecture supports:

    2 MiB huge pages

The bootloader uses large mappings during early startup.

The kernel currently uses standard 4 KiB mappings for runtime allocations.

---

## 5. PT (Level 1)

The Page Table is the final level of translation.

Each entry maps:

    One 4 KiB page

This is the most frequently modified level during normal operation.

Current users include:

    User-page mappings
    User stacks
    LAPIC mappings
    ELF segment mappings

---

## 6. Entry Format

Each x86_64 page-table entry contains:

    Physical address
    Permission flags
    State flags
    Optional NX flag

Primary layout:

    Bits 0-11
        Flags

    Bits 12-51
        Physical address

    Bits 52-62
        Reserved / architecture-defined

    Bit 63
        NO_EXECUTE

---

## 7. Common Flags

Bulldog currently uses the following page-table flags.

### PRESENT

Meaning:

    Page exists and is valid

Used for:

    All active mappings

---

### WRITABLE

Meaning:

    Write access allowed

Used for:

    Kernel memory
    User stacks
    Writable ELF segments

---

### USER_ACCESSIBLE

Meaning:

    User mode may access the mapping

Used for:

    User program pages
    User stacks
    User ELF segments

This flag is critical for Ring 3 execution.

---

### NO_EXECUTE

Meaning:

    Instruction fetches prohibited

Used for:

    LAPIC MMIO mappings
    Non-executable ELF segments
    Data pages

Bulldog currently makes active use of NX protection.

---

### HUGE_PAGE

Meaning:

    Large page mapping

Current status:

    Supported by hardware
    Used by bootloader startup mappings
    Not currently managed by kernel runtime code

---

## 8. Current Kernel Usage

Common mapping classes:

### Kernel Code

Typically:

    PRESENT

Executable code remains non-user-accessible.

---

### Kernel Data

Typically:

    PRESENT
    WRITABLE
    NO_EXECUTE

---

### LAPIC MMIO

Typically:

    PRESENT
    WRITABLE
    NO_EXECUTE

---

### User Program Pages

Typically:

    PRESENT
    USER_ACCESSIBLE

or:

    PRESENT
    WRITABLE
    USER_ACCESSIBLE

depending on segment permissions.

---

## 9. Kernel vs User Mappings

### Kernel Mappings

Characteristics:

    Higher-half addresses
    Not user accessible
    Shared globally

Examples:

    Kernel code
    Kernel heap
    HHDM
    LAPIC mappings

---

### User Mappings

Current implementation:

    User pages share the active address space
    User pages are marked USER_ACCESSIBLE
    User stacks are dynamically allocated

Examples:

    ELF segments
    User stacks

Note:

    Per-process page tables are not yet implemented.

---

## 10. Page Table Creation

Current architecture:

    Bootloader creates page tables
            ↓
    Bootloader loads CR3
            ↓
    Kernel inherits structures
            ↓
    Kernel attaches via OffsetPageTable

The kernel does not currently construct the initial paging hierarchy.

Instead it extends and manages existing mappings.

---

## 11. OffsetPageTable Integration

Bulldog accesses page tables through:

    OffsetPageTable

This abstraction provides:

    Safe traversal
    Mapping creation
    Mapping updates
    Address translation

Current users include:

    map_page()
    map_user_page()
    map_user_segment()
    map_lapic_mmio()

---

## 12. USER_ACCESSIBLE Page Walk

Before entering Ring 3:

    force_user_access()

walks the page-table hierarchy.

Levels visited:

    PML4
    PDPT
    PD
    PT

The function ensures:

    USER_ACCESSIBLE

is set throughout the translation chain.

This guarantees successful user-mode memory access.

---

## 13. Address Translation Example

Translation flow:

    Virtual Address
            ↓
    PML4 lookup
            ↓
    PDPT lookup
            ↓
    PD lookup
            ↓
    PT lookup
            ↓
    Physical Address

The HHDM allows the kernel to access these page-table structures through normal virtual addresses.

---

## 14. Safety Invariants

The following assumptions must remain true:

    Kernel mappings must not be user accessible
    HHDM mappings must remain valid
    Page tables must remain 4 KiB aligned
    Page-table entries must remain internally consistent
    User mappings must be tracked for cleanup
    PAGE_PRESENT must be set on active mappings

Violating these assumptions will destabilize memory management.

---

## 15. Current Limitations

Not yet implemented:

    Per-process page tables
    Copy-on-write
    Demand paging
    Shared memory
    Guard pages
    Automatic page reclamation
    Address-space isolation

These features remain future work.

---

## 16. Future Enhancements

Planned future work:

    Per-process address spaces
    Page-table debugging tools
    Page-table dumping utilities
    Demand paging
    Large-page management
    Shared pages
    KASLR support

Current architecture is intentionally simple until scheduler and virtual-memory work begins.

---

## Contributor Notes

- Bootloader owns initial page-table creation.
- Kernel owns runtime page-table management.
- USER_ACCESSIBLE pages must be verified before entering Ring 3.
- HHDM assumptions must remain synchronized with bootloader paging.
- Page-table modifications should be reviewed alongside paging.md and hhdm.md.
- User-memory support depends heavily on correct page-table permissions.

---

## License

MIT or Apache 2.0 — TBD.