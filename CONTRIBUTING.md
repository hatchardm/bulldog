# Contributing to Bulldog

Bulldog is a custom operating system kernel written in Rust for the `x86_64-bulldog`
architecture. Contributions are welcome from anyone interested in systems programming,
low‑level architecture, Rust internals, or kernel design.

This document describes the contributor workflow, project hygiene, subsystem boundaries,
and expectations for submitting changes.

---

## Development Workflow

Bulldog provides a helper script at `scripts/dev` to streamline common tasks:

```
./scripts/dev build    # Compile the kernel
./scripts/dev run      # Launch Bulldog in QEMU
./scripts/dev test     # Execute kernel tests
./scripts/dev clean    # Remove build artifacts
```

The script checks prerequisites (Rust nightly, required components, QEMU availability) and
confirms when your environment is ready.

All contributions should build and run successfully using this script.

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
└── README.md               # Project overview
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

Contributors should keep documentation in both layers up to date.

---

## Branching Strategy

Bulldog’s development is organized around milestone branches:

```
main                    → Latest stable kernel build (APIC baseline)
feature/pic8259         → Legacy PIC baseline
feature/apic            → APIC + LAPIC timer milestone
feature/syscall         → Privilege switching + syscall infrastructure
feature/kernel-init-rebuild → Current milestone (kernel init, interface layer)
```

Contributor branches should use:

- `feat/<name>` for new features  
- `fix/<name>` for bug fixes  
- `doc/<name>` for documentation updates  

All work should merge into the active milestone branch before reaching `main`.

---

## Commit Guidelines

- Keep commits atomic and descriptive  
- Use imperative mood:  
  - `Add syscall dispatcher`  
  - `Refactor GDT initialization`  
- Reference issues when applicable  
- Document unsafe blocks with clear justification  
- Avoid large, multi‑purpose commits  
- Ensure commit history remains readable and contributor‑friendly

Example:

```
feat(syscall): add alloc/free trampolines and update syscall table
```

---

## Code Style

- Follow Rust’s `rustfmt` defaults  
- Avoid `println!` for runtime output  
- Use the kernel logging interface (`info!`, `warn!`, `error!`, `debug!`, `trace!`)  
- Keep subsystem boundaries clean  
- Maintain architectural clarity  
- Avoid hidden side effects  
- Prefer explicit, deterministic behavior  
- Keep unsafe usage minimal and well‑documented

---

## Testing

Before submitting a PR:

- Run `./scripts/dev test`  
- Ensure tests pass under QEMU  
- Validate syscall entry/exit paths  
- Confirm register preservation (syscall harness)  
- Ensure logs remain deterministic  
- Avoid introducing nondeterministic behavior in interrupts or scheduling

Hardware validation is optional but encouraged if APIC support is available.

---

## Documentation Requirements

All contributions that modify subsystem behavior must update the relevant documentation:

### Architecture changes → update:
- `docs/architecture/*`

### Interface changes → update:
- `docs/interface/*`

Documentation must:

- Be clear and technically accurate  
- Explain invariants and constraints  
- Describe contributor expectations  
- Match the current implementation  
- Avoid outdated references  

Bulldog’s documentation is part of the project’s architecture — treat it as a first‑class component.

---

## Roadmap Alignment

Bulldog’s roadmap includes:

### Near-Term
- Kernel init rebuild  
- Syscall validation  
- Minimal user‑mode execution  
- ELF loader  
- Basic VFS (ramfs/devfs)

### Mid-Term
- Cooperative scheduler  
- Preemptive scheduling via LAPIC timer  
- Process creation syscalls  
- FD table integration  
- User-mode heap allocator

### Long-Term
- Signals  
- IPC  
- Multi-threading  
- Multi-CPU load balancing  
- ext2/ext4 filesystem support  
- Dynamic linker / shared libraries

Contributions should align with these milestones or propose new directions via issues.

---

## PR Checklist

- [ ] Code builds via `scripts/dev build`  
- [ ] Kernel runs via `scripts/dev run`  
- [ ] Tests pass under QEMU  
- [ ] Syscall harness passes  
- [ ] Commit messages follow guidelines  
- [ ] Documentation updated  
- [ ] No stray `println!` in runtime code  
- [ ] No architectural regressions  
- [ ] No nondeterministic behavior introduced  

---

## Getting Help

- Open an issue for bugs, questions, or feature requests  
- Use discussions for design proposals  
- Keep PRs small, focused, and reviewable  
- Ask maintainers for subsystem guidance when needed  

---

## Logging Guidelines

Bulldog uses the `log` crate for all runtime output. This ensures consistent severity levels,
structured messages, and contributor‑friendly debugging.

Use:

- `info!` for normal runtime events  
- `debug!` / `trace!` for verbose developer details  
- `warn!` for unexpected but survivable conditions  
- `error!` for fatal faults or unrecoverable errors  

`println!` is allowed **only** during early boot before `logger_init()` runs.

---

## License

Contributions are welcome under MIT or Apache 2.0 (TBD).

---

Thanks for helping make Bulldog robust, maintainable, and contributor‑friendly!
