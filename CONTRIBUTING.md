# 🤝 Contributing to Bulldog

Thanks for your interest in Bulldog! This project is a custom operating system kernel written in Rust, targeting the `x86_64-bulldog` architecture. Contributions are welcome from anyone curious about systems programming, Rust internals, or low‑level OS concepts.

---

## 🛠 Development Workflow

Bulldog provides a helper script at `scripts/dev` to streamline common tasks:

```bash
./scripts/dev build   # Compile the kernel
./scripts/dev run     # Launch Bulldog in QEMU
./scripts/dev test    # Execute kernel tests
./scripts/dev clean   # Remove build artifacts
```

This script checks prerequisites (Rust nightly, required components, VS Code extensions if applicable) and confirms when your environment is ready.

---

## 📂 Project Hygiene

- **Branching**  
  - Use `main` for stable development.  
  - Create feature branches for experiments or new subsystems.  
  - Prefix branches with `feat/`, `fix/`, or `doc/` for clarity.

- **Commits**  
  - Keep commits atomic and descriptive.  
  - Use imperative mood: `Add syscall scaffolding`, not `Added syscall scaffolding`.  
  - Reference issues when applicable.

- **Code Style**  
  - Follow Rust’s `rustfmt` defaults.  
  - Avoid `println!` for kernel output — use the logging interface (coming soon).  
  - Document unsafe blocks with clear justification.

---

## 🧪 Testing

- Run `./scripts/dev test` before submitting a PR.  
- Kernel tests should be deterministic and reproducible.  
- For experimental features, mark tests clearly and isolate them in separate modules.

---

## 📜 Guidelines

- Be explicit in documentation: assume contributors are new to kernel development.  
- Provide confirmation messages in scripts to reduce ambiguity.  
- Keep onboarding frictionless — scripts should auto‑check prerequisites and fail gracefully.  
- Contributions are welcome under MIT or Apache 2.0 (TBD).

---

## 🚀 Roadmap Alignment

Bulldog’s roadmap includes:

- Privilege switching  
- Syscall interface  
- Process scheduling  
- User mode execution  

When contributing, align your work with these milestones or propose new directions via issues.

---

## 🐾 Getting Help

- Open an issue for bugs, questions, or feature requests.  
- Use discussions for design proposals.  
- PRs should be small, focused, and reviewed before merge.

---

Logging Guidelines
Bulldog uses the log crate for all runtime output. This ensures consistent severity levels, structured messages, and contributor-friendly debugging.

✅ Use log macros
Normal runtime events → info!

Verbose developer details → debug! or trace!

Unexpected but survivable conditions → warn!

Fatal faults or unrecoverable errors → error!

⚠️ print!/println! macros
These macros are only for early boot stages (before logger_init() runs).

They bypass the logging system and write directly to the framebuffer.

Do not use them for runtime output once the logger is initialized.

🚫 Do not
Introduce new println! calls in runtime code.

Use println! as a shortcut for logging — always prefer the appropriate log macro.

Thanks for helping make Bulldog robust, maintainable, and contributor‑friendly!
