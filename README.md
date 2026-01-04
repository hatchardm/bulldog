# Bulldog Kernel – SYSCALL Development Branch

Bulldog is a custom operating system kernel written in Rust for the `x86_64-bulldog`
architecture. It is built from scratch with a focus on safety, reliability, and architectural
clarity. The project explores low-level OS concepts such as paging, interrupt handling,
privilege switching, and syscall infrastructure.

This branch focuses on privilege switching, syscall development, and user ↔ kernel transitions.

---

## Branch Roadmap

```
main                → Latest stable kernel build (APIC baseline)
│
├── feature/pic8259 → Legacy PIC interrupt controller
│
├── feature/apic    → APIC milestone (paging, LAPIC timer, vector hygiene)
│
└── feature/syscall → Privilege switching + syscall infrastructure
```

---

## Getting Started

### Prerequisites

To build Bulldog, you will need:

- A nightly Rust compiler  
- The `llvm-tools-preview` component  
- QEMU (recommended) or real hardware with APIC support  

Install the required Rust component:

```bash
rustup component add llvm-tools-preview
```

---

## Build Instructions

Clone the repository:

```bash
git clone https://github.com/hatchardm/bulldog.git
cd bulldog
```

Build the kernel:

```bash
cargo build -Z bindeps
```

Run in QEMU:

```bash
qemu-system-x86_64 \
  -kernel target/x86_64-bulldog/debug/bulldog \
  -serial stdio \
  -smp 2 \
  -enable-kvm
```

---

## Compatibility Notes

### loc_api Nightly Feature Fix

If you encounter a build error in `loc_api` related to the deprecated `const_fn` feature:

```rust
#![cfg_attr(feature = "nightly", feature(const_fn))]
```

Replace it with:

```rust
#![cfg_attr(feature = "nightly", feature(const_fn_trait_bound))]
```

Apply this fix at line 91 of `loc_api/lib.rs`.  
Ensure your `Cargo.toml` enables the nightly feature.

---

## Syscall Development Overview

## Architecture Documentation

For detailed subsystem documentation, see:

- docs/architecture/README.md – Architecture index  
- docs/privilege-switching.md – Ring 0 ↔ Ring 3 transitions  
- docs/syscall.md – Syscall development guide  
- docs/syscall-table.md – Syscall numbering and table  
- docs/syscall-harness-guide.md – Syscall test harness  


This branch introduces:

- Privilege switching  
  - Ring 0 ↔ Ring 3 transitions via GDT/TSS  
  - Proper stack switching on interrupts and exceptions  

- Syscall infrastructure  
  - Initial syscall table and dispatcher  
  - Example syscalls for testing  

- Contributor visibility  
  - Logging of syscall entry, arguments, and return values  
  - Minimal user ↔ kernel test harness  

See the following documents for details:

- `docs/syscall.md` – Syscall development guide  
- `docs/privilege-switching.md` – Privilege switching mechanics  
- `docs/syscall-table.md` – Syscall table and numbering  
- `docs/syscall-harness-guide.md` – Syscall test harness  

---

## Current Syscalls

| Number | Name       | Status        | Notes                          |
|--------|------------|---------------|--------------------------------|
| 1      | sys_write  | Implemented   | Writes buffer to fd            |
| 2      | sys_exit   | Implemented   | Terminates process             |
| 3      | sys_open   | Implemented   | Opens file descriptor          |
| 4      | sys_read   | Stub example  | Reads buffer (to be implemented) |

See `docs/syscall-table.md` for details on adding new syscalls.

---

## Roadmap

- Paging and memory management  
- Interrupt handling and IST setup  
- GDT/TSS initialization  
- APIC interrupt controller integration  
- Privilege switching  
- Syscall interface and dispatcher  
- Process scheduling  
- User mode execution  

---

## Branching Strategy

Bulldog’s development is organized around feature branches that represent major architectural
milestones:

| Branch          | Purpose / Stage                                |
|-----------------|-------------------------------------------------|
| main            | Latest integrated kernel (APIC baseline)        |
| feature/pic8259 | Legacy PIC interrupt controller                 |
| feature/apic    | LAPIC/APIC interrupt controller development     |
| feature/syscall | Privilege switching + syscall infrastructure    |

New features should be developed in a dedicated `feature/*` branch and merged into `main`
once complete.

---

## Contributing

Bulldog is designed for open-source collaboration. Contributors interested in kernel
development, Rust internals, or low-level architecture are welcome.

### Contributor Workflow

- Keep commits atomic and descriptive  
- Document unsafe blocks with justification  
- Test under QEMU before submitting pull requests  
- Align contributions with roadmap milestones  
- Ensure all syscalls log entry, arguments, and return values  
- Update `docs/syscall.md` and `docs/syscall-table.md` when adding new syscalls  

Additional contributor guidelines will be expanded as the project evolves.

---

## License

MIT or Apache 2.0 — to be determined. Contributions are welcome under either license.

---

## Disclaimer

Bulldog and its subsystems (syscalls, APIC, PIC8259, paging, and related features) are
experimental and provided “as is” without warranty of any kind. They are intended for
research, learning, and contributor experimentation. Running Bulldog on real hardware may
expose quirks or limitations. Use at your own risk. By contributing or running Bulldog, you
agree to abide by the project license.




