# 🧠 MARKOSPRINT: A Rust-Based Kernel

MARKOSPRINT is a custom operating system kernel written in Rust, focused on safety, modularity, and low-level control. It’s built from scratch with paging, interrupt handling, and privilege separation in mind.

---

## 🚀 Project Goals

- Build a robust kernel using Rust’s safety guarantees  
- Implement paging, GDT/TSS, and interrupt handling  
- Transition to user mode with syscall scaffolding  
- Lay groundwork for process scheduling and multitasking  
- Create a contributor-friendly open-source platform  

---

## 🧩 Architecture Overview

- **Memory Management**: Paging enabled with custom page table setup  
- **GDT/TSS**: Global Descriptor Table and Task State Segment configured for privilege switching  
- **IDT & Interrupts**: Interrupt Descriptor Table with macro-based handler generation  
- **Bootloader**: Custom bootloader setup using `bootimage` or `cargo-xbuild`  
- **Fault Recovery**: Defensive programming and catch-all interrupt handlers  

---

## 🛠️ Build & Run Instructions

Ensure you're using nightly Rust:

```bash
rustup override set nightly
Then build and run with:

bash
cargo run -Z bindeps
This command automatically launches QEMU and boots the kernel. No manual QEMU invocation required.

🧪 Compatibility Notes
🔧 loc_api Nightly Feature Fix
If loc_api throws an error related to the deprecated const_fn feature:

rust
#![cfg_attr(feature = "nightly", feature(const_fn))]
Replace it with:

rust
#![cfg_attr(feature = "nightly", feature(const_fn_trait_bound))]
This fix should be applied at line 91 of loc_api/lib.rs. It resolves build errors on newer nightly Rust versions where const_fn has been removed.

🌱 Roadmap
[ ] Syscall interface and user mode transition

[ ] Process scheduler and context switching

[ ] Basic file system scaffolding

[ ] Contributor guide and branching strategy

📜 License
This project is licensed under the MIT License.
