use core::sync::atomic::{AtomicU64, Ordering};
use log::{info, error};

pub static TICKS: AtomicU64 = AtomicU64::new(0);

pub fn tick() {
    TICKS.fetch_add(1, Ordering::Relaxed);
}

pub fn get_ticks() -> u64 {
    TICKS.load(Ordering::Relaxed)
}

/// Logs every `interval` ticks for proof of life.
pub fn health_check(interval: u64) {
    let t = get_ticks();
    if t % interval == 0 {
        info!("Health check: Kernel alive, ticks={}", t);
    }
}

/// A stateful watchdog that tolerates startup and only errors on real stalls.
pub struct Watchdog {
    last_ticks: u64,
    window: u64,
    grace_left: u32,
    consecutive_failures: u32,
    failure_threshold: u32,
}
impl Watchdog {
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

    /// Silent unless a stall persists beyond grace and threshold.
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
                    // Hard stop only on sustained stall.
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





