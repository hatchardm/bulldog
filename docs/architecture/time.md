# Bulldog Kernel – Timekeeping Subsystem (Current Implementation)

> **Disclaimer:**  
> This document describes Bulldog’s *current* timekeeping subsystem exactly as implemented today.  
> It does **not** include future plans such as calibrated timers, sleep APIs, scheduler integration,  
> or high‑resolution timekeeping.  
>  
> For related subsystems, see:  
> - `timer.md` (LAPIC timer configuration)  
> - `interrupts.md` (timer interrupt handler)  
> - `kernel_init.md` (initialization sequence)

Bulldog’s time subsystem is intentionally minimal.  
It provides a global tick counter incremented by the LAPIC timer interrupt, along with simple diagnostic helpers.

---

# 1. Overview

The time subsystem currently provides:

- A global 64‑bit tick counter  
- A `tick()` function called on each LAPIC timer interrupt  
- A periodic `health_check()` logger  
- A stateful `Watchdog` for detecting stalled tick progression  

There is **no scheduler**, **no sleep API**, **no calibration**, and **no real timekeeping** yet.

---

# 2. Global Tick Counter

The tick counter is defined as:

```rust
pub static TICKS: AtomicU64 = AtomicU64::new(0);
```

Properties:

- 64‑bit atomic integer  
- Incremented once per LAPIC timer interrupt  
- Represents an abstract “tick” unit  
- Not tied to milliseconds or seconds  
- Not calibrated  

This counter is the foundation for future timekeeping and scheduling.

---

# 3. Tick Function

The LAPIC timer interrupt handler calls:

```rust
pub fn tick() {
    TICKS.fetch_add(1, Ordering::Relaxed);
}
```

Characteristics:

- Uses `Relaxed` ordering (sufficient for a monotonic counter)  
- Very fast — designed for use inside an interrupt handler  
- No logging (to avoid log spam)  
- No overflow handling (wraparound is acceptable for now)  

---

# 4. Reading the Tick Counter

The current tick count can be retrieved with:

```rust
pub fn get_ticks() -> u64 {
    TICKS.load(Ordering::Relaxed)
}
```

This is used by:

- Diagnostics  
- Health checks  
- Watchdog logic  
- Future scheduler integration  

---

# 5. Health Check

The `health_check()` function provides a simple “proof of life” mechanism:

```rust
pub fn health_check(interval: u64) {
    let t = get_ticks();
    if t % interval == 0 {
        info!("Health check: Kernel alive, ticks={}", t);
    }
}
```

Behavior:

- Logs once every `interval` ticks  
- Useful during early bring‑up  
- Confirms that timer interrupts are firing  
- Confirms that logging is operational  

This is not a watchdog — it is purely diagnostic.

---

# 6. Watchdog

The `Watchdog` struct provides a tick‑based stall detector:

```rust
pub struct Watchdog {
    last_ticks: u64,
    window: u64,
    grace_left: u32,
    consecutive_failures: u32,
    failure_threshold: u32,
}
```

### Purpose

- Detects when the kernel stops making progress  
- Useful for catching deadlocks or interrupt starvation  
- Panics only after grace and failure thresholds are exceeded  

### Operation

```rust
pub fn check(&mut self) {
    let current = get_ticks();

    if current < self.last_ticks + self.window {
        return;
    }

    if current == self.last_ticks {
        if self.grace_left > 0 {
            self.grace_left -= 1;
        } else {
            self.consecutive_failures += 1;
            if self.consecutive_failures >= self.failure_threshold {
                panic!("Watchdog timeout: ticks stalled");
            }
        }
    } else {
        self.last_ticks = current;
        self.consecutive_failures = 0;
    }
}
```

### Notes

- The watchdog is **not** automatically invoked  
- It must be driven manually by the kernel (e.g., from a future scheduler loop)  
- It is safe to use even before SMP or user‑mode support  

---

# 7. Integration With LAPIC Timer

The LAPIC timer interrupt handler calls `tick()`:

```rust
extern "x86-interrupt" fn lapic_timer_handler(...) {
    tick();
    send_eoi();
}
```

This ties the time subsystem directly to the LAPIC timer frequency.

See `timer.md` for details.

---

# 8. Limitations (Current)

The time subsystem currently lacks:

- Real timekeeping  
- Tick calibration  
- Tick‑to‑millisecond conversion  
- Sleep/delay APIs  
- Per‑CPU tick counters  
- Scheduler integration  
- High‑resolution timers  
- TSC‑deadline mode  

These will be added incrementally.

---

# 9. Future Work

Planned enhancements include:

- LAPIC calibration using PIT or HPET  
- Converting ticks to real time  
- Per‑CPU tick counters for SMP  
- Tickless idle  
- High‑resolution timers  
- Integration with a real scheduler  
- Timeouts and sleep primitives  

These features will be documented once implemented.

---

# License

MIT or Apache 2.0 — TBD.