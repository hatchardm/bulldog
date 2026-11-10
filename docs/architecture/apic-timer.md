## Bulldog Kernel – APIC Timer

Reviewed: November 2026

This document describes Bulldog's current LAPIC timer implementation, interrupt delivery model, and future scheduler integration plans.

It complements:

    apic.md
    apic-implementation.md
    interrupts.md
    cpu-init.md

---

## 1. Overview

Bulldog currently uses the Local APIC timer as its primary periodic interrupt source.

Current responsibilities:

    Kernel timekeeping
    Tick generation
    Interrupt verification

Future responsibilities include:

    Preemptive scheduling
    Sleep primitives
    Timing services
    Profiling

The scheduler is not currently integrated with timer interrupts.

---

## 2. Initialization Sequence

Current initialization flow:

    Map LAPIC MMIO
            ↓
    Enable LAPIC
            ↓
    Configure divisor
            ↓
    Configure timer vector
            ↓
    Select periodic mode
            ↓
    Load initial count
            ↓
    Begin timer operation

This sequence occurs during:

    setup_apic()

---

## 3. Current Configuration

Current timer settings:

    Mode          = Periodic
    Vector        = 0x31
    Divisor       = 16
    Initial Count = 500000

These values are currently fixed.

Dynamic calibration has not yet been implemented.

---

## 4. Timer Modes

### One-Shot

Interrupt fires once when the count reaches zero.

Supported by hardware.

Not currently used.

---

### Periodic

Interrupt fires repeatedly at a fixed interval.

Current Bulldog mode.

Used for:

    Timekeeping
    Tick generation

---

### TSC-Deadline

Interrupt fires when the timestamp counter reaches a programmed target.

Not currently implemented.

Future work.

---

## 5. Current Calibration Status

Dynamic calibration is not currently implemented.

The timer presently uses a fixed configuration:

    Divisor       = 16
    Initial Count = 500000

Future revisions may introduce calibration using:

    PIT
    HPET
    TSC

or other timing sources.

---

## 6. Interrupt Delivery

Current timer interrupt vector:

    0x31

Current handler:

    lapic_timer_handler()

Execution flow:

    LAPIC Interrupt
            ↓
    IDT Vector 0x31
            ↓
    IST1 Stack
            ↓
    tick()
            ↓
    send_eoi()
            ↓
    Return

The handler remains intentionally lightweight.

---

## 7. IST Integration

The timer executes using:

    IST1

Current rationale:

    Fault isolation
    Reliable interrupt processing
    Reduced dependency on interrupted stacks

IST1 is shared with:

    Page Fault handling

as documented in:

    gdt.md

---

## 8. Timekeeping

The timer currently contributes to:

    Kernel tick tracking

through:

    tick()

The timer serves as Bulldog's current time source infrastructure.

Additional timing services remain future work.

---

## 9. Scheduler Integration

Scheduler integration is planned but not currently implemented.

Future timer responsibilities may include:

    Scheduler tick generation
    Time-slice management
    Context-switch triggers
    Process accounting

Current timer interrupts do not perform scheduling operations.

---

## 10. Future Enhancements

Planned improvements:

    Dynamic timer calibration
    TSC-deadline mode
    Per-CPU timers
    High-resolution timing
    Sleep queues
    Scheduler integration
    Performance profiling
    Advanced timekeeping services

These capabilities are not currently implemented.

---

## Contributor Notes

- The timer currently operates in periodic mode.
- Preserve vector 0x31 assignments unless architecture documentation is updated.
- Timer handlers must send EOI before returning.
- Keep timer handlers lightweight.
- Scheduler integration should build on the existing timer infrastructure.

---

## License

MIT or Apache 2.0 — TBD.