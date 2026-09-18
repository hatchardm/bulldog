# Bulldog Kernel – APIC Implementation Details

This document describes the implementation details of Bulldog’s APIC subsystem, including
LAPIC MMIO access, register programming, spurious interrupt configuration, and timer setup.

It complements:

- `lapic.md` (architecture)
- `apic-timer.md`
- `interrupts-implementation.md`

---

## 1. Overview

Bulldog uses the Local APIC (LAPIC) for:

- Interrupt routing
- Timer interrupts
- Spurious interrupt handling
- End‑of‑interrupt signaling

The legacy PIC is disabled early in boot.

---

## 2. LAPIC MMIO Mapping

The LAPIC is memory‑mapped at:

```
0xFEE00000
```

Bulldog maps this region into the higher‑half direct map (HHDM) using:

- `PRESENT`
- `WRITABLE`
- `NO_EXECUTE`

Access is performed using volatile reads/writes.

---

## 3. LAPIC Registers Used

Bulldog currently uses:

- `SVR` – Spurious interrupt vector + LAPIC enable
- `LVT_TIMER` – Timer mode + vector
- `TIMER_INIT_COUNT` – Initial timer count
- `TIMER_CURRENT_COUNT` – Current timer count
- `TIMER_DIVIDE` – Timer divisor
- `EOI` – End‑of‑interrupt register

Future work will use:

- `ICR` – Inter‑processor interrupts
- `LVT_ERROR` – LAPIC error reporting

---

## 4. LAPIC Enable

Bulldog enables LAPIC by writing:

- Spurious vector (`0xFF`)
- LAPIC enable bit (bit 8)

This activates LAPIC interrupt delivery.

---

## 5. LAPIC Timer Setup

Bulldog configures the timer in one‑shot mode:

1. Set divisor
2. Set initial count
3. Set vector (`0x31`)
4. Enable timer interrupt

The timer handler:

- Calls `tick()`
- Sends EOI
- Runs on IST1

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

## 8. Future Enhancements

- TSC‑deadline mode
- Per‑CPU LAPIC initialization
- Multi‑CPU interrupt routing
- LAPIC error handling
- High‑resolution timers
- IPIs for SMP

---

## License
MIT or Apache 2.0 — TBD.
