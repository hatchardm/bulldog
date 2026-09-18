# Bulldog Kernel – Modern x86_64 Rust Kernel

Bulldog is a custom operating system kernel written in Rust for the `x86_64-bulldog`
architecture. It is built from scratch with a focus on clarity, safety, architectural
discipline, and contributor accessibility.

Bulldog explores low‑level OS concepts including:

- Higher‑half kernel design  
- Paging and memory management  
- GDT/TSS initialization  
- Interrupt handling and vector hygiene  
- Privilege switching (Ring 3 ↔ Ring 0)  
- Syscall interface and dispatcher  
- Syscall harness and ABI validation  
- Early user‑mode execution  
- Planned VFS and ELF loader subsystems  

This repository contains both the kernel source and the full documentation tree.

---

## Repository Structure

```
bulldog/
│
├── kernel/                 # Kernel source code
│   ├── src/
│   └── Cargo.toml
│
├── docs/
│   ├── architecture/       # Hardware-facing subsystem documentation
│   └── interface/          # User-facing kernel interface documentation
│
└── README.md               # This file
```

### Architecture Layer (`docs/architecture/`)
Hardware‑facing subsystems:

- Memory & paging  
- GDT/TSS  
- Interrupts & IDT  
- APIC/LAPIC  
- Kernel layout  
- Boot & initialization  
- Context switching (CPU-level)  

### Interface Layer (`docs/interface/`)
User‑mode boundary and kernel interface:

- Syscall ABI  
- Syscall table  
- Syscall dispatcher  
- Syscall harness  
- VFS design (planned)  
- ELF loader (planned)  
- Process model (interface-level)  
- Privilege switching (user ↔ kernel transitions)

This separation keeps Bulldog contributor‑friendly and easy to navigate.

---

## Current Development Branch

The active milestone branch is:

```
feature/kernel-init-rebuild
```

This branch focuses on:

- Rebuilding kernel initialization  
- Cleaning subsystem boundaries  
- Rewriting text/console and framebuffer integration  
- Validating syscall entry/exit paths  
- Preparing for user‑mode execution  
- Establishing the interface documentation tree  

Previous milestone branches:

```
main                → Latest stable kernel build (APIC baseline)
feature/pic8259     → Legacy PIC baseline
feature/apic        → APIC + LAPIC timer milestone
feature/syscall     → Privilege switching + syscall infrastructure
```

---

## Getting Started

### Prerequisites

You will need:

- Nightly Rust toolchain  
- `llvm-tools-preview` component  
- QEMU (recommended) or real hardware with APIC support  

Install required Rust components:

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

## Syscall Interface Overview

Bulldog implements a simple, stable syscall ABI:

| Number | Name   | Handler Function           | Notes |
|--------|--------|----------------------------|-------|
| 1      | write  | `sys_write`                | FD write |
| 2      | exit   | `sys_exit_trampoline`      | Process exit |
| 3      | open   | `sys_open`                 | Returns FD |
| 4      | read   | `sys_read`                 | FD read |
| 5      | alloc  | `sys_alloc_trampoline`     | Allocates memory |
| 6      | free   | `sys_free_trampoline`      | Frees memory |
| 7      | close  | `sys_close_trampoline`     | Closes FD |

See:

- `docs/interface/syscall.md`  
- `docs/interface/syscall-table.md`  
- `docs/interface/syscall-harness-guide.md`  
- `docs/interface/dispatcher.md`

---

## Roadmap

### Near-Term
- Finalize kernel init rebuild  
- Validate syscall entry/exit paths  
- Establish minimal user‑mode execution  
- Implement ELF loader  
- Implement basic VFS (ramfs/devfs)

### Mid-Term
- Cooperative scheduler  
- Preemptive scheduling via LAPIC timer  
- Process creation syscalls  
- FD table integration with VFS  
- User-mode heap allocator

### Long-Term
- Signals  
- IPC  
- Multi-threading  
- Multi-CPU load balancing  
- ext2/ext4 filesystem support  
- Dynamic linker / shared libraries

---

## Contributing

Bulldog is designed for open-source collaboration. Contributors interested in kernel
development, Rust internals, or low-level architecture are welcome.

### Contributor Workflow

- Keep commits atomic and descriptive  
- Document unsafe blocks with justification  
- Test under QEMU before submitting pull requests  
- Align contributions with roadmap milestones  
- Update relevant documentation in `docs/architecture/` or `docs/interface/`  
- Ensure syscall harness tests remain deterministic  

---

## License

MIT or Apache 2.0 — to be determined.

---

## Disclaimer

Bulldog and its subsystems (paging, interrupts, APIC, syscalls, VFS, ELF loader, and related
features) are experimental and provided “as is” without warranty of any kind. They are
intended for research, learning, and contributor experimentation. Running Bulldog on real
hardware may expose quirks or limitations. Use at your own risk.



