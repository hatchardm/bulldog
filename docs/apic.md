# Bulldog OS – APIC Documentation

This document provides technical details for the **APIC milestone** in Bulldog OS.  
It explains the Local APIC (LAPIC) and I/O APIC configuration, register layouts, and handler logic used in the `feature/apic` branch.

---

## 📌 Overview

Bulldog transitioned from the legacy PIC8259 interrupt controller to the **Advanced Programmable Interrupt Controller (APIC)** system.  
This milestone introduces:
- LAPIC timer configuration in periodic mode
- End-of-interrupt (EOI) handling
- I/O APIC IRQ routing
- Logger integration in interrupt handlers
- Kernel heartbeat via watchdog loop

---

## 🖥️ Local APIC (LAPIC)

### Key Registers
- **ID Register (0x20)** → Identifies the LAPIC.
- **Version Register (0x30)** → Reports LAPIC version and max LVT entries.
- **Spurious Interrupt Vector Register (0xF0)** → Enables LAPIC and sets spurious vector.
- **LVT Timer Register (0x320)** → Configures timer mode and vector.
- **Initial Count Register (0x380)** → Sets starting value for timer.
- **Current Count Register (0x390)** → Decrements in real time.
- **Divide Configuration Register (0x3E0)** → Sets timer divisor.

### Timer Configuration
- Mode: **Periodic**
- Vector: Assigned in `interrupts.rs` (e.g., `0x20` for timer IRQ).
- Divisor: Typically set to 16 for predictable tick rate.
- EOI: Always written after handling an interrupt to acknowledge completion.

---

## 🌐 I/O APIC

### Responsibilities
- Routes external IRQs (keyboard, disk, NIC) to LAPIC vectors.
- Provides masking/unmasking for selective device interrupts.

### Key Registers
- **IOREGSEL (0x00)** → Selects register index.
- **IOWIN (0x10)** → Read/write selected register.
- **Redirection Table (0x10–0x3F)** → Maps IRQs to vectors.

### Example Mapping
- IRQ0 (Timer) → Vector `0x20`
- IRQ1 (Keyboard) → Vector `0x21`
- IRQ14 (Primary ATA) → Vector `0x2E`

---

## 🧩 Interrupt Handler Logic

### General Flow
1. CPU receives interrupt vector.
2. Handler logs event using `logger.rs`.
3. Handler performs device-specific action (if applicable).
4. LAPIC EOI register written to acknowledge completion.
5. Control returns to kernel.

### Logger Integration
- Color-coded output for visibility.
- Deadlock-free design using safe primitives.
- Avoids recursive lock acquisition inside handlers.

---

## 🩺 Health Check & Watchdog

- Implemented in `time.rs`.
- Periodic LAPIC timer ticks act as kernel heartbeat.
- Contributors can verify kernel liveness by observing tick output in QEMU console.

---

## 🛠 Contributor Notes

- Always send EOI after handling LAPIC interrupts.
- Keep vector assignments consistent across LAPIC and I/O APIC.
- Document new IRQ mappings in this file for clarity.
- Test changes in QEMU before running on hardware.

---

## 📜 References

- Intel® 64 and IA-32 Architectures Software Developer’s Manual, Vol. 3 (System Programming Guide).
- OSDev Wiki: [APIC](https://wiki.osdev.org/APIC)

---


## Disclaimer

Bulldog and its subsystems (including syscalls, APIC, PIC8259, paging, and related features)  
are experimental and provided "as is" without warranty of any kind. They are intended for  
research, learning, and contributor experimentation. Running Bulldog on real hardware may  
expose quirks or limitations. Use at your own risk. The maintainers and contributors are  
not liable for any damages or issues arising from its use. By contributing or running Bulldog,  
you agree to abide by the terms of the project license.
