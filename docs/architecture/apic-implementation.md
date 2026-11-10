## Bulldog Kernel – APIC Implementation Details

Reviewed: November 2026

This document describes the implementation details of Bulldog's APIC subsystem, including LAPIC MMIO access, register programming, spurious interrupt configuration, and timer setup.

It complements:

    apic.md
    apic-timer.md
    interrupts-implementation.md

---

## 1. Overview

Bulldog uses the Local APIC (LAPIC) for:

    Interrupt routing
    Timer interrupts
    Spurious interrupt handling
    End-of-interrupt signaling

The legacy PIC is disabled early during kernel initialization.

Current implementation targets:

    Bootstrap Processor (BSP)

IOAPIC and SMP functionality remain future work.

---

## 2. LAPIC MMIO Mapping

The LAPIC physical base address is:

    0xFEE00000

Bulldog maps this region into higher-half virtual memory using:

    PRESENT
    WRITABLE
    NO_EXECUTE

Current mapping is performed by:

    map_lapic_mmio()

before:

    setup_apic()

is executed.

All LAPIC access occurs through volatile MMIO operations.

---

## 3. LAPIC Registers Used

Bulldog currently uses:

    SVR
    ID
    VERSION
    LVT_TIMER
    DIVIDE_CONFIG
    INITIAL_COUNT
    CURRENT_COUNT
    EOI

Current responsibilities:

    LAPIC enablement
    Identity reporting
    Timer configuration
    End-of-interrupt signaling

Future work may additionally use:

    ICR
    LVT_ERROR

for SMP and advanced interrupt functionality.

---

## 4. LAPIC Enable

Bulldog enables the LAPIC through:

    SVR

Current configuration:

    Vector: 0xFF
    Enable Bit: Set

Equivalent behavior:

    0x100 | 0xFF

This activates Local APIC interrupt delivery.

---

## 5. LAPIC Timer Configuration

Bulldog currently configures the LAPIC timer in:

    Periodic Mode

Current settings:

    Vector        = 0x31
    Divisor       = 16
    Initial Count = 500000

Configuration sequence:

    Configure divisor
            ↓
    Configure vector
            ↓
    Select periodic mode
            ↓
    Load initial count
            ↓
    Start timer

The timer vector is defined by:

    LAPIC_TIMER_VECTOR

in the interrupt subsystem.

---

## 6. LAPIC Timer Handler

Current timer handler:

    lapic_timer_handler()

Behavior:

    tick()
            ↓
    send_eoi()

Characteristics:

    Runs on IST1
    No verbose logging
    Generates kernel timer ticks

The scheduler does not currently consume these timer events.

---

## 7. End-of-Interrupt (EOI)

EOI is written through:

    send_eoi()

which performs:

    lapic_write(EOI, 0)

Every LAPIC-delivered interrupt must acknowledge completion using EOI.

Failure to do so prevents subsequent interrupt delivery.

---

## 8. Spurious Interrupts

Current vector:

    0xFF

Handler:

    spurious_handler()

Behavior:

    Log interrupt
            ↓
    Send EOI

This vector is also used during APIC enablement.

---

## 9. APIC Identity Verification

Current initialization verifies:

    LAPIC ID
    LAPIC Version
    CPUID APIC ID

The following values are logged:

    LAPIC MMIO ID
    CPUID-reported APIC ID
    LAPIC Version Register

This information assists debugging and future SMP development.

---

## 10. Interaction With Interrupts

The interrupt subsystem provides:

    LAPIC_TIMER_VECTOR
    IDT entries
    IST stack routing

Current vector assignments:

    LAPIC Timer -> 0x31
    Spurious    -> 0xFF

The LAPIC timer executes on:

    IST1

for increased reliability.

---

## 11. Future Enhancements

Planned improvements:

    IOAPIC initialization
    Hardware IRQ routing
    SMP support
    Inter-processor interrupts
    LAPIC error handling
    Dynamic timer calibration
    Scheduler integration
    High-resolution timers

These features are not currently implemented.

---

## Contributor Notes

- Ensure map_lapic_mmio() executes before setup_apic().
- Preserve EOI handling in LAPIC interrupt paths.
- Keep LAPIC diagnostic logging intact.
- Maintain synchronization with interrupts.md and apic.md.
- Document new vectors whenever LAPIC functionality expands.

---

## License

MIT or Apache 2.0 — TBD.