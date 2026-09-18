# Bulldog Kernel – APIC Timer

This document describes Bulldog’s APIC timer subsystem: how the LAPIC timer is configured,
how interrupts are delivered, and how the timer integrates with scheduling and timekeeping.

It complements:

- `apic.md`
- `interrupts.md`
- `time.md`
- `scheduler.md` (interface)

---

## 1. Overview

The Local APIC (LAPIC) provides a per‑CPU timer capable of generating periodic interrupts.
Bulldog uses the LAPIC timer for:

- Future preemptive scheduling  
- Timekeeping  
- Sleep/wake primitives  
- Profiling  

---

## 2. Initialization Steps

1. Map LAPIC MMIO region  
2. Enable LAPIC  
3. Configure timer divisor  
4. Load initial count  
5. Set vector  
6. Enable interrupts  

Bulldog currently configures the timer in one‑shot mode for calibration.

---

## 3. Timer Modes

### One‑Shot  
Interrupt fires once when count reaches zero.

### Periodic  
Interrupt fires repeatedly at a fixed interval.

### TSC‑Deadline (future)  
Interrupt fires when TSC reaches a programmed value.

Bulldog will use periodic mode for preemptive scheduling.

---

## 4. Calibration

Calibration determines:

- Timer frequency  
- Tick duration  
- Conversion between LAPIC ticks and real time  

Bulldog uses PIT or HPET (future) as a reference.

---

## 5. Interrupt Delivery

Timer interrupts are delivered to:

```
vector = TIMER_VECTOR
```

The handler:

- Acknowledges interrupt  
- Updates timekeeping state  
- Triggers scheduler tick (future)  

---

## 6. Integration with Scheduler

Future Bulldog versions will:

- Trigger a scheduler tick on each timer interrupt  
- Save current process context  
- Select next runnable process  
- Perform context switch  

This enables preemptive multitasking.

---

## 7. Future Enhancements

- TSC‑deadline mode  
- Per‑CPU timers  
- High‑resolution timers  
- Sleep queues  
- Profiling hooks  
- Timekeeping API  

---

## License
MIT or Apache 2.0 — to be determined.
