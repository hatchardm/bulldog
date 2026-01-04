# Bulldog Kernel – Timer Subsystem

This document describes Bulldog’s LAPIC‑based timer subsystem, including LAPIC timer configuration, interrupt flow, the global tick counter, and diagnostic mechanisms such as the health check and watchdog. It serves as a reference for contributors working on scheduling, interrupt handling, and future timekeeping features.

---

## Overview

Bulldog uses the Local APIC (LAPIC) timer as its primary timing source. The LAPIC timer operates in periodic mode and generates a regular stream of interrupts that advance the global tick counter and drive the scheduler. The legacy PIC8259 and PIT are not used for periodic interrupts.

The timer subsystem provides:

- LAPIC enable and configuration  
- Periodic timer interrupts  
- Global tick counter maintenance  
- Scheduler tick notifications  
- Health check and watchdog support  

---

## LAPIC Mapping

The LAPIC MMIO region is mapped into the higher‑half kernel address space at a fixed virtual base. All LAPIC registers are accessed through this virtual mapping. The physical LAPIC base is obtained from MSR 0x1B and must be mapped before the LAPIC is enabled.

---

## LAPIC Initialization

### Base MSR

The LAPIC base MSR (0x1B) provides the physical LAPIC base address and the LAPIC enable bit. If the enable bit is clear, the kernel halts.

### LAPIC ID and Version

The LAPIC ID and version registers are read during initialization. The CPUID‑reported APIC ID is also logged to confirm consistency between the LAPIC mapping and the CPU’s view of the APIC.

### Enabling the LAPIC

The LAPIC is enabled by writing to the Spurious Interrupt Vector Register (SVR). Bit 8 enables the LAPIC, and vector 0xFF is used for spurious interrupts.

---

## LAPIC Timer Configuration

### Registers

The LAPIC timer uses the LVT Timer register, the divide configuration register, the initial count register, the current count register, and the EOI register.

### Mode and Divisor

Bulldog configures the LAPIC timer in periodic mode. The divide configuration is set to produce a divisor of 16. The timer interrupt vector is assigned from the kernel’s vector layout and resides in the APIC‑specific interrupt range.

### Initial Count

The LAPIC timer is loaded with a fixed initial count of 500000. This value is not calibrated. The resulting tick frequency depends on the LAPIC timer clock and may vary across systems. The current countdown value can be inspected through the current count register for debugging and future calibration work.

---

## Timer Interrupt Flow

When the LAPIC timer fires:

1. The CPU enters the interrupt stub for the LAPIC timer vector.  
2. The stub saves registers and transfers control to the timer handler.  
3. The handler increments the global tick counter.  
4. The scheduler is notified of the tick.  
5. The LAPIC EOI register is written.  

The timer interrupt path is intentionally minimal to keep interrupt latency low.

---

## Global Tick Counter

Bulldog maintains a single global tick counter that increments on each LAPIC timer interrupt. The counter is implemented using an atomic 64‑bit integer to ensure correctness under concurrent updates from multiple CPUs.

A tick represents one LAPIC timer interrupt. It is an abstract time unit and is not currently tied to milliseconds or seconds. Higher‑level timekeeping will be introduced once calibration is implemented.

---

## Health Check

The health check mechanism logs a periodic liveness message based on the global tick counter. This provides a simple heartbeat during early development and helps confirm that timer interrupts are firing, the kernel is making forward progress, and logging is operational.

---

## Watchdog

The watchdog monitors kernel progress over tick‑based windows. It tracks the last observed tick count, the expected window size, a grace period for missed windows, and a failure counter. If ticks do not advance for several consecutive windows after grace is exhausted, the watchdog triggers a kernel panic.

This mechanism detects interrupt starvation, scheduler stalls, and deadlocks that halt tick progression.

---

## Scheduling Integration

The scheduler is invoked on each timer tick. It may charge CPU time to the current thread, decrement time slices, trigger preemption, and perform periodic housekeeping. Bulldog currently uses a global tick counter, providing a unified timebase across CPUs.

---

## Limitations and Future Work

Current limitations:

- No calibration of the LAPIC timer  
- Tick frequency is hardware‑dependent  
- Sleep durations (future) will be approximate  
- Per‑CPU LAPIC timer initialization for APs is not yet documented  
- No high‑resolution or one‑shot timers  

Future work:

- LAPIC calibration using PIT or HPET  
- Tick‑to‑millisecond conversion helpers  
- Per‑CPU tick counters  
- Tickless idle  
- TSC‑deadline mode  
- High‑resolution timers  

---

## License

MIT or Apache 2.0 — to be determined.

