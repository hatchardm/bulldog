use core::sync::atomic::{AtomicU64, Ordering};
use log::info;

/// Global tick counter incremented by the LAPIC timer handler.
/// Provides a simple heartbeat for the kernel.
pub static TICKS: AtomicU64 = AtomicU64::new(0);

/// Increment the global tick counter.
/// Called on each LAPIC timer interrupt.
pub fn tick() {
    TICKS.fetch_add(1, Ordering::Relaxed);
}

/// Return the current tick count.
pub fn get_ticks() -> u64 {
    TICKS.load(Ordering::Relaxed)
}

/// Periodic health check.
/// Logs a "proof of life" message every `interval` ticks.
pub fn health_check(interval: u64) {
    let t = get_ticks();
    if t % interval == 0 {
        info!("Health check: Kernel alive, ticks={}", t);
    }
}

/// A stateful watchdog that monitors kernel progress.
/// - `window`: tick interval to check for progress.
/// - `grace_left`: number of tolerated missed windows before counting failures.
/// - `failure_threshold`: number of consecutive failures before panic.
pub struct Watchdog {
    last_ticks: u64,
    window: u64,
    grace_left: u32,
    consecutive_failures: u32,
    failure_threshold: u32,
}

impl Watchdog {
    /// Create a new watchdog with the given parameters.
    /// Starts with the current tick count as baseline.
    pub fn new(window: u64, grace_checks: u32, failure_threshold: u32) -> Self {
        let t = get_ticks();
        Self {
            last_ticks: t,
            window,
            grace_left: grace_checks,
            consecutive_failures: 0,
            failure_threshold,
        }
    }

    /// Check kernel progress.
    /// - If ticks have advanced within the window, reset failures.
    /// - If no progress, decrement grace or increment failures.
    /// - Panic only if failures exceed threshold after grace is exhausted.
    pub fn check(&mut self) {
        let current = get_ticks();

        // Not yet at window boundary: do nothing.
        if current < self.last_ticks + self.window {
            return;
        }

        // Window reached: evaluate progress.
        if current == self.last_ticks {
            // No progress within window.
            if self.grace_left > 0 {
                self.grace_left -= 1;
            } else {
                self.consecutive_failures += 1;
                if self.consecutive_failures >= self.failure_threshold {
                    panic!("Watchdog timeout: ticks stalled");
                }
            }
        } else {
            // Progress observed: advance baseline and clear failures.
            self.last_ticks = current;
            self.consecutive_failures = 0;
        }
    }
}






