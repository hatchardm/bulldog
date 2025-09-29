# 🐾 Bulldog Kernel

**Bulldog** is a custom operating system kernel written in Rust, targeting the `x86_64-bulldog` architecture. It’s built from scratch with a focus on safety, reliability, and architectural clarity. This project explores low-level OS concepts such as paging, interrupt handling, privilege switching, and syscall scaffolding.

---

## 🚀 Getting Started

### Prerequisites

To build Bulldog, you’ll need:

- A **nightly Rust compiler**
- The `llvm-tools-preview` component


Install the required Rust component:

```bash
rustup component add llvm-tools-preview


bash



🛠 Build Instructions
Clone the repo:

bash
git clone https://github.com/hatchardm/bulldog.git
cd bulldog

Build the kernel:

bash
cargo build -Z bindeps

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

Contributor guidelines

Branching strategies for experimental features

📜 License
MIT or Apache 2.0 — TBD. Contributions welcome under either license.
