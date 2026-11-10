## Bulldog Kernel – Local APIC (LAPIC)

Reviewed: November 2026

This document describes Bulldog's current Local APIC (LAPIC) architecture.

It complements:

    apic.md
    apic-implementation.md
    apic-timer.md
    interrupts.md
    gdt.md

This document describes the current implementation rather than future APIC plans.

---

## 1. Overview

Bulldog currently uses the Local APIC (LAPIC) as its primary interrupt controller.

Current responsibilities:

    Interrupt delivery
    Periodic timer interrupts
    Spurious interrupt handling
    End-of-interrupt signaling
    APIC identification
    APIC version reporting

Current implementation targets:

    Bootstrap Processor (BSP)

IOAPIC and SMP support are not currently implemented.

---

## 2. LAPIC MMIO Mapping

Current LAPIC physical address:

    0xFEE00000

Bulldog maps the LAPIC into kernel virtual memory using:

    map_lapic_mmio()

Current mapping:

    Physical:
        0xFEE00000

    Virtual:
        LAPIC_VIRT_BASE

Current flags:

    PRESENT
    WRITABLE
    NO_EXECUTE

The LAPIC must be mapped before:

    setup_apic()

is executed.

---

## 3. LAPIC Registers

Bulldog currently uses:

    ID
    VERSION
    SVR
    LVT_TIMER
    DIVIDE_CONFIG
    INITIAL_COUNT
    CURRENT_COUNT
    EOI

These registers support all currently implemented LAPIC functionality.

---

## 4. LAPIC Enablement

During initialization Bulldog:

    Reads APIC MSR
            ↓
    Verifies LAPIC enabled
            ↓
    Enables SVR
            ↓
    Configures timer
            ↓
    Starts periodic interrupts

Current SVR configuration:

    Enable Bit = Set
    Vector     = 0xFF

This activates LAPIC interrupt delivery.

---

## 5. LAPIC Identification

Current diagnostics record:

    LAPIC ID
    LAPIC Version
    CPUID APIC ID

These values are logged during:

    setup_apic()

to assist debugging and future SMP development.

---

## 6. LAPIC Timer

Current configuration:

    Mode          = Periodic
    Vector        = 0x31
    Divisor       = 16
    Initial Count = 500000

Current handler:

    tick()
            ↓
    send_eoi()

The timer executes on:

    IST1

for improved reliability.

---

## 7. End-of-Interrupt (EOI)

LAPIC interrupts are completed using:

    send_eoi()

Current implementation:

    lapic_write(EOI, 0)

EOI must be issued before returning from LAPIC interrupt handlers.

---

## 8. Spurious Interrupts

Current vector:

    0xFF

Current handler:

    spurious_handler()

Behavior:

    Log interrupt
            ↓
    Send EOI

This vector is also used by SVR configuration.

---

## 9. Interrupt Integration

The LAPIC currently integrates with:

    IDT
    GDT
    TSS
    IST1 stack

Current vector assignments:

    LAPIC Timer -> 0x31
    Spurious    -> 0xFF

The LAPIC timer executes using:

    LAPIC_IST_INDEX

defined by the TSS.

---

## 10. Current Limitations

Not currently implemented:

    IOAPIC
    Inter-processor interrupts
    LAPIC error handling
    SMP startup
    Per-CPU APIC configuration

These remain future work.

---

## 11. Future Enhancements

Planned work:

    IOAPIC initialization
    SMP support
    Inter-processor interrupts
    Dynamic timer calibration
    TSC-deadline mode
    High-resolution timers
    Per-CPU timer configuration

---

## Contributor Notes

- Map LAPIC MMIO before setup_apic().
- Preserve EOI handling.
- Keep vector assignments synchronized with interrupts.rs.
- Preserve setup_apic() diagnostics.
- Document new LAPIC vectors when added.

---

## License

MIT or Apache 2.0 — TBD.