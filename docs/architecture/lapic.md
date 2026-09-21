# Bulldog Kernel – APIC / LAPIC Architecture

This document describes Bulldog’s APIC architecture, including the Local APIC (LAPIC),
interrupt routing, timer configuration, and MMIO mapping. It complements:

- `apic-timer.md`
- `interrupts.md`
- `memory/hhdm.md`

---

## 1. Overview

The Advanced Programmable Interrupt Controller (APIC) provides:

- Per‑CPU interrupt routing  
- LAPIC timer  
- Spurious interrupt handling  
- Inter‑processor interrupts (IPI)  
- Error reporting  

Bulldog uses the LAPIC exclusively; the legacy PIC is disabled.

---

## 2. LAPIC MMIO Mapping

The LAPIC is memory‑mapped at:

```
0xFEE00000
```

Bulldog maps this region into the higher‑half direct map (HHDM) during early initialization.

Access is performed via volatile MMIO operations.

---

## 3. LAPIC Registers

Key registers:

- `SVR` – Spurious interrupt vector  
- `LVT_TIMER` – Timer configuration  
- `TIMER_INIT_COUNT` – Initial timer count  
- `TIMER_CURRENT_COUNT` – Current timer count  
- `TIMER_DIVIDE` – Timer divisor  
- `EOI` – End‑of‑interrupt register  
- `ICR` – Inter‑processor interrupt control  

Bulldog writes these registers directly via MMIO.

---

## 4. LAPIC Enable

LAPIC is enabled by setting:

- Bit 8 of `SVR`  
- Spurious vector (0xFF)  

This activates LAPIC interrupt delivery.

---

## 5. LAPIC Timer

The LAPIC timer is configured in:

- One‑shot mode (current)  
- Periodic mode (future)  
- TSC‑deadline mode (future)

Bulldog uses vector:

```
0x31
```

The timer interrupt handler:

- Increments kernel tick  
- Sends EOI  
- Uses IST for reliability  

---

## 6. End‑of‑Interrupt (EOI)

EOI is written to:

```
0xB0
```

Bulldog wraps this in:

```rust
send_eoi();
```

This acknowledges the interrupt and allows new interrupts to be delivered.

---

## 7. Spurious Interrupts

Spurious interrupts use vector:

```
0xFF
```

Handler logs and acknowledges the interrupt.

---

## 8. Inter‑Processor Interrupts (IPI)

Future Bulldog versions will use IPIs for:

- Multi‑CPU startup  
- TLB shootdown  
- Cross‑CPU scheduling  
- Kernel debugger  

---

## 9. Future Enhancements

- TSC‑deadline mode  
- Per‑CPU LAPIC initialization  
- Multi‑CPU interrupt routing  
- LAPIC error handling  
- High‑resolution timers  

---

## License
MIT or Apache 2.0 — to be determined.
