# 🐾 Bulldog Kernel – APIC Development Branch

**Bulldog** is a custom operating system kernel written in Rust, targeting the `x86_64-bulldog` architecture.  
It’s built from scratch with a focus on safety, reliability, and architectural clarity. This project explores low-level OS concepts such as paging, interrupt handling, privilege switching, and syscall scaffolding.

This branch, **`feature/apic`**, represents the milestone where Bulldog transitions from the legacy PIC8259 interrupt controller to the **Local APIC (LAPIC)** and **I/O APIC** infrastructure.

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

## 🖥️ APIC Milestone Overview

This branch introduces:

- LAPIC timer configuration
  - Periodic mode setup with correct vector hygiene.
  - End-of-interrupt (EOI) handling validated.
- Interrupt routing via I/O APIC
  - Clean mapping of IRQs to vectors.
  - Mask/unmask logic for selective device interrupts.
- Logger integration in interrupt handlers
  - Visible, color-coded output for debugging.
  - Deadlock-free registration using safe primitives.
- Health check & watchdog loop
  - Kernel heartbeat visible via timer ticks.
  - Contributors can instantly verify kernel liveness.

---

## 🧭 Roadmap

- [x] Paging and memory management  
- [x] Interrupt handling and IST setup  
- [x] GDT/TSS initialization  
- [x] `loc_api` fix and memory map alignment  
- [x] APIC interrupt controller integration  
- [ ] Privilege switching  
- [ ] Syscall interface  
- [ ] Process scheduling  
- [ ] User mode execution  

---

## 🌱 Branching Strategy

Bulldog’s development is organized around feature branches that act as benchmarks of the OS’s evolution:

| Branch                     | Purpose / Benchmark Stage |
|-----------------------------|---------------------------|
| main                       | Latest integrated kernel (APIC-based) |
| feature/pic8259            | Legacy PIC interrupt controller solution |
| feature/apic               | LAPIC/APIC interrupt controller development |
| feature/privilege-switching| Ring transitions, privilege level switching |
| feature/syscall-interface  | System call ABI and dispatcher |

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
