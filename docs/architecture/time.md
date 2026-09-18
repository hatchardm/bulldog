# Bulldog Kernel – Timekeeping and LAPIC Timer (Current Implementation)

This document describes Bulldog’s *current* timekeeping model.  
It reflects the real behavior of:

- `time.rs` (tick counter)
- `apic.rs` (LAPIC timer configuration)
- `interrupts.rs` (LAPIC timer handler)
- `kernel_init` (timer initialization sequence)

Bulldog does **not** yet implement high‑resolution timers, RTC integration, wall‑clock time,
or scheduler time slices. This document describes exactly what exists today.

---

## 1. Overview

Bulldog’s timekeeping is currently based on a **single global tick counter** incremented by
the LAPIC timer interrupt.

Today, Bulldog provides:

- A global tick counter (`tick_count`)
- A `tick()` function called by the LAPIC timer interrupt
- A periodic LAPIC timer configured during `setup_apic()`
- A stable, deterministic tick frequency (fixed divisor + initial count)

There is **no**:

- Real‑time clock (RTC) integration  
- HPET support  
- TSC calibration  
- Wall‑clock time  
- Sleep queues  
- Per‑process timers  
- Scheduler time slices  

These will be added later.

---

## 2. Global Tick Counter

`time.rs` defines:

```rust
static TICK_COUNT: AtomicU64 = AtomicU64::new(0);

pub fn tick() {
    TICK_COUNT.fetch_add(1, Ordering::SeqCst);
}

pub fn ticks() -> u64 {
    TICK_COUNT.load(Ordering::SeqCst)
}
```

This counter increments once per LAPIC timer interrupt.

### Properties

- Monotonic  
- Global  
- Atomic  
- Not tied to real time  
- Not calibrated  

Ticks are used for:

- Logging  
- Basic timing tests  
- Future scheduler integration  

---

## 3. LAPIC Timer Configuration

The LAPIC timer is configured in `setup_apic()`:

- Divisor: `0b0011` (divide by 16)
- Mode: `Periodic`
- Initial count: `500_000`
- Vector: `LAPIC_TIMER_VECTOR`

This produces a stable periodic interrupt rate on most hardware.

The LAPIC timer handler:

```rust
extern "x86-interrupt" fn lapic_timer_handler(_stack_frame: InterruptStackFrame) {
    tick();
    send_eoi();
}
```

This is the only source of time progression in Bulldog today.

---

## 4. Tick Frequency

Bulldog does **not** currently calibrate the LAPIC timer.

Tick frequency depends on:

- LAPIC bus clock  
- Divisor (16)  
- Initial count (500,000)  

This means tick frequency varies between machines.

Future work will introduce:

- TSC calibration  
- LAPIC timer calibration  
- HPET fallback  
- Deterministic tick rate  

---

## 5. Planned Timekeeping Features

Bulldog will eventually support:

### High‑Resolution Time

- TSC calibration  
- Nanosecond resolution  
- Monotonic clock  
- Wall‑clock time via RTC  

### Scheduler Integration

- Time slices  
- Sleep queues  
- Timeouts  
- Per‑process timers  

### Kernel APIs

- `sleep(ms)`  
- `deadline()`  
- `now()`  
- `uptime()`  

### Hardware Support

- HPET  
- ACPI PM timer  
- APIC timer calibration  
- Per‑CPU timers (SMP)

None of these are implemented yet.

---

## 6. Contributor Notes

- The tick counter is the only time source today  
- LAPIC timer must be initialized before enabling interrupts  
- Tick frequency is not calibrated — do not assume real‑time accuracy  
- Timekeeping will expand significantly once syscalls and user‑mode are online  
- Keep this document updated as new timing features are added  

---

## License

MIT or Apache 2.0 — TBD.
