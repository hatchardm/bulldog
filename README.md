# 🐾 Bulldog Kernel – SYSCALL Development Branch

**Bulldog** is a custom operating system kernel written in Rust, targeting the `x86_64-bulldog` architecture.  
It’s built from scratch with a focus on safety, reliability, and architectural clarity. This project explores low-level OS concepts such as paging, interrupt handling, privilege switching, and syscall scaffolding.

This branch focuses on **privilege switching, syscall infrastructure, and user ↔ kernel transitions**.

---

## 🗺️ Bulldog Kernel Branch Roadmap

main                → Latest stable kernel build (currently APIC baseline)
│
├── feature/pic8259 → Preserved legacy branch (original PIC8259 interrupt controller)
│
├── feature/apic    → APIC milestone (includes paging, LAPIC timer, vector hygiene)
│
└── feature/syscall → Active development branch (privilege switching + syscall infrastructure)

---

## 🚀 Getting Started

### Prerequisites

To build Bulldog, you’ll need:

- A **nightly Rust compiler**
- The `llvm-tools-preview` component
- QEMU (recommended) or real hardware with APIC support

Install the required Rust component:

rustup component add llvm-tools-preview

---

## 🛠 Build Instructions

Clone the repo:

git clone https://github.com/hatchardm/bulldog.git
cd bulldog

Build the kernel:

cargo build -Z bindeps

Run in QEMU:

qemu-system-x86_64 \
  -kernel target/x86_64-bulldog/debug/bulldog \
  -serial stdio \
  -smp 2 \
  -enable-kvm

---

## 🧪 Compatibility Notes

### 🔧 `loc_api` Nightly Feature Fix
If you're using the nightly Rust toolchain and encounter a build error in `loc_api` related to the deprecated `const_fn` feature:

#![cfg_attr(feature = "nightly", feature(const_fn))]

Replace it with:

#![cfg_attr(feature = "nightly", feature(const_fn_trait_bound))]

📍 Apply this fix at **line 91 of `loc_api/lib.rs`**.  
It resolves build errors on newer nightly Rust versions where `const_fn` has been removed in favor of `const_fn_trait_bound`.

Ensure your `Cargo.toml` enables the nightly feature.

---

## 🖥️ Syscall Development Overview

This branch introduces:

- Privilege switching
  - Ring 0 ↔ Ring 3 transitions via GDT/TSS setup.
  - Proper stack switching on interrupts/exceptions.
- Syscall infrastructure
  - Initial syscall table and dispatcher.
  - Example syscall (e.g. framebuffer write) for testing.
- Contributor visibility
  - Logging of syscall invocations.
  - Minimal user ↔ kernel test harness.

---

## 🧭 Roadmap

- [x] Paging and memory management  
- [x] Interrupt handling and IST setup  
- [x] GDT/TSS initialization  
- [x] APIC interrupt controller integration  
- [ ] Privilege switching  
- [ ] Syscall interface  
- [ ] Process scheduling  
- [ ] User mode execution  

---

## 🌱 Branching Strategy

Bulldog’s development is organized around feature branches that act as benchmarks of the OS’s evolution:

| Branch          | Purpose / Benchmark Stage                        |
|-----------------|--------------------------------------------------|
| main            | Latest integrated kernel (APIC-based)            |
| feature/pic8259 | Legacy PIC interrupt controller solution         |
| feature/apic    | LAPIC/APIC interrupt controller development      |
| feature/syscall | Privilege switching + syscall infrastructure     |

Contributors can check out any feature branch to explore Bulldog at that stage.  
New features should be developed in their own `feature/*` branch, then merged into `main` once complete.

---

## 🤝 Contributing

Bulldog is designed with open-source collaboration in mind.  
If you're interested in kernel development, Rust internals, or low-level architecture, we’d love your input.

Coming soon:
- Expanded documentation  
- Contributor guidelines  
- Branching strategies for experimental features  

---

## 📜 License

MIT or Apache 2.0 — TBD. Contributions welcome under either license.

