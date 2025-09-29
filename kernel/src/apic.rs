
pub mod apic {
    use core::ptr::write_volatile;
    use crate::interrupts::LAPIC_TIMER_VECTOR;

    const LAPIC_BASE: usize = 0xFEE00000;

    #[repr(usize)]
    enum LapicRegister {
        EOI = 0xB0,
        SVR = 0xF0,
        LVT_TIMER = 0x320,
        INITIAL_COUNT = 0x380,
        CURRENT_COUNT = 0x390,
        DIVIDE_CONFIG = 0x3E0,
    }

    fn lapic_write(reg: LapicRegister, value: u32) {
        unsafe {
            let reg_ptr = (LAPIC_BASE + reg as usize) as *mut u32;
            write_volatile(reg_ptr, value);
        }
    }

    pub fn init() {
    // Enable LAPIC
    lapic_write(LapicRegister::SVR, 0x100 | LAPIC_TIMER_VECTOR as u32);

    // Set divide config (e.g., divide by 16)
    lapic_write(LapicRegister::DIVIDE_CONFIG, 0b0011);

    // Set timer mode to periodic
    lapic_write(LapicRegister::LVT_TIMER, 0x200 | LAPIC_TIMER_VECTOR as u32);

    // Set initial count (e.g., 10 million ticks)
    lapic_write(LapicRegister::INITIAL_COUNT, 10_000_000);

    lapic_write(LapicRegister::LVT_TIMER, LAPIC_TIMER_VECTOR as u32);
}



    pub fn send_eoi() {
        lapic_write(LapicRegister::EOI, 0);
    }
}
