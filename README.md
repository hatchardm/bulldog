# 🐾 Bulldog Kernel

**Bulldog** is a custom operating system kernel written in Rust, targeting the `x86_64-bulldog` architecture. It’s built from scratch with a focus on safety, reliability, and architectural clarity. This project explores low-level OS concepts such as paging, interrupt handling, privilege switching, and syscall scaffolding.

---

## 🚀 Getting Started

### Prerequisites

To build Bulldog, you’ll need:

- A **nightly Rust compiler**
- The `llvm-tools-preview` component
- A cross-compilation target: `x86_64-bulldog`

Install the required Rust component:

```bash
rustup component add llvm-tools-preview

Set up the target architecture:

bash
rustup target add x86_64-bulldog

🧰 Optional: Binary Inspection Tools
To inspect and disassemble the kernel binary, install cargo-binutils:

bash
cargo install cargo-binutils

Then use:

bash
cargo objdump -- -d target/x86_64-bulldog/debug/kernel
cargo size -- target/x86_64-bulldog/debug/kernel
These commands let you view disassembly and symbol sizes without manually locating LLVM binaries.

🛠 Build Instructions
Clone the repo:

bash
git clone https://github.com/hatchardm/bulldog.git
cd bulldog

Build the kernel:

bash
cargo build --target x86_64-bulldog

🧪 Compatibility Notes
🔧 loc_api Nightly Feature Fix
If you're using the nightly Rust toolchain and encounter a build error in loc_api related to the deprecated const_fn feature:

rust
#![cfg_attr(feature = "nightly", feature(const_fn))]

Replace it with:

rust
#![cfg_attr(feature = "nightly", feature(const_fn_trait_bound))]
📍 Apply this fix at line 91 of loc_api/lib.rs. It resolves build errors on newer nightly Rust versions where const_fn has been removed in favor of const_fn_trait_bound.

Ensure your Cargo.toml enables the nightly feature:

toml
[features]
nightly = []
📚 Project Structure
src/ — Kernel source code

arch/ — Architecture-specific setup (GDT, TSS, paging, etc.)

boot/ — Bootloader and entry point

docs/ — Documentation and design notes

.gitignore — Cleaned for Rust and kernel artifacts

🧭 Roadmap
[x] Paging and memory management

[x] Interrupt handling and IST setup

[x] GDT/TSS initialization

[x] loc_api fix and memory map alignment

[ ] Privilege switching

[ ] Syscall interface

[ ] Process scheduling

[ ] User mode execution

🤝 Contributing
Bulldog is designed with open-source collaboration in mind. If you're interested in kernel development, Rust internals, or low-level architecture, we’d love your input.

Coming soon:

Expanded documentation

## Disclaimer

Bulldog and its subsystems (including syscalls, APIC, PIC8259, paging, and related features)  
are experimental and provided "as is" without warranty of any kind. They are intended for  
research, learning, and contributor experimentation. Running Bulldog on real hardware may  
expose quirks or limitations. Use at your own risk. The maintainers and contributors are  
not liable for any damages or issues arising from its use. By contributing or running Bulldog,  
you agree to abide by the terms of the project license.


Contributor guidelines

Branching strategies for experimental features

📜 License
MIT or Apache 2.0 — TBD. Contributions welcome under either license.
