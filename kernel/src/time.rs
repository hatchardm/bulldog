use core::sync::atomic::{AtomicU64, Ordering};

static TICKS: AtomicU64 = AtomicU64::new(0);

/// Called by the LAPIC timer interrupt handler
pub fn tick() {
    let t = TICKS.fetch_add(1, Ordering::Relaxed);
}


/// Returns the current tick count
pub fn get_ticks() -> u64 {
    TICKS.load(Ordering::Relaxed)
}
