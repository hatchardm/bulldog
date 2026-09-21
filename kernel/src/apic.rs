use core::ptr::{read_volatile, write_volatile};
use crate::interrupts::LAPIC_TIMER_VECTOR;
use log::{info, debug};
use core::arch::asm;

/// Virtual base address where the LAPIC is memory-mapped.
/// This is mapped into the higher-half kernel space.
pub const LAPIC_VIRT_BASE: u64 = 0xFFFF_FF00_0000_0000;
pub const LAPIC_BASE: usize = LAPIC_VIRT_BASE as usize;

/// Spurious interrupt vector used when enabling the LAPIC.
pub const SPURIOUS_VECTOR: u32 = 0xFF;

/// LAPIC timer modes (encoded in bits 17–18 of LVT_TIMER).
#[repr(u32)]
pub enum LapicTimer {
    OneShot     = 0b00 << 17,
    Periodic    = 0b01 << 17,
    TscDeadline = 0b10 << 17,
}

/// LAPIC register offsets (relative to LAPIC base).
#[repr(usize)]
pub enum LapicRegister {
    LVT_TIMER     = 0x320, // Local Vector Table: Timer Mode + Vector
    DIVIDE_CONFIG = 0x3E0, // Timer divisor
    INITIAL_COUNT = 0x380, // Initial timer count
    CURRENT_COUNT = 0x390, // Current timer count
    EOI           = 0xB0,  // End-of-interrupt register
    SVR           = 0xF0,  // Spurious Interrupt Vector Register
    ID            = 0x20,  // LAPIC ID
    VERSION       = 0x30,  // LAPIC version
}

/// Read a 32-bit value from a LAPIC register.
pub fn lapic_read(reg: LapicRegister) -> u32 {
    unsafe {
        let reg_ptr = (LAPIC_VIRT_BASE + reg as u64) as *mut u32;
        read_volatile(reg_ptr)
    }
}

/// Write a 32-bit value to a LAPIC register.
pub fn lapic_write(reg: LapicRegister, value: u32) {
    unsafe {
        let reg_ptr = (LAPIC_VIRT_BASE + reg as u64) as *mut u32;
        debug!("lapic_write → VIRT {:#x}", reg_ptr as usize);
        write_volatile(reg_ptr, value);
    }
}

/// Configure and enable the Local APIC.
/// - Reads LAPIC base MSR and verifies LAPIC is enabled.
/// - Logs LAPIC version and ID.
/// - Enables LAPIC via SVR with spurious vector.
/// - Configures timer in periodic mode with divisor 16.
/// - Sets initial count and confirms configuration.
pub fn setup_apic() {
    #[cfg(not(feature = "syscall_tests"))]
    {info!("Entered setup_apic()");}

    // Read LAPIC base MSR (0x1B).
    let apic_base = read_msr(0x1B);
    let base_phys = apic_base & 0xFFFFF000;
    let enabled = (apic_base >> 11) & 1;
    #[cfg(not(feature = "syscall_tests"))]
    {info!("APIC base MSR: {:#x}", apic_base);
    info!("APIC physical base: {:#x}", base_phys);
    info!("LAPIC enabled: {}", enabled);}
    if enabled == 0 {
        panic!("LAPIC is not enabled!");
    }

    // LAPIC version and ID.
    let version = lapic_read(LapicRegister::VERSION);
    #[cfg(not(feature = "syscall_tests"))]
    {info!("LAPIC VERSION: {:#x}", version);}

    let id = lapic_read(LapicRegister::ID);
    let cpuid_id = cpuid_apic_id();
    #[cfg(not(feature = "syscall_tests"))]
    {info!("LAPIC ID: {:#x}, CPUID APIC ID: {:#x}", id, cpuid_id);}

    // Enable LAPIC via Spurious Interrupt Vector Register.
    lapic_write(LapicRegister::SVR, 0x100 | SPURIOUS_VECTOR);
    #[cfg(not(feature = "syscall_tests"))]
    {info!("SVR written (enable + spurious=0xFF)");}

    // Configure LAPIC timer: divisor = 16, periodic mode.
    lapic_write(LapicRegister::DIVIDE_CONFIG, 0b0011);
    lapic_write(
        LapicRegister::LVT_TIMER,
        LAPIC_TIMER_VECTOR as u32 | LapicTimer::Periodic as u32,
    );

    // Confirm mode + vector.
    let lvt = lapic_read(LapicRegister::LVT_TIMER);
    #[cfg(not(feature = "syscall_tests"))]
    {info!(
        "LVT_TIMER: {:#x} (periodic bit set? {})",
        lvt,
        (lvt & (1 << 17)) != 0
    );}

    // Set initial count (tick rate tuning).
    lapic_write(LapicRegister::INITIAL_COUNT, 500_000);

    let current = lapic_read(LapicRegister::CURRENT_COUNT);
    #[cfg(not(feature = "syscall_tests"))]
    {info!("LAPIC CURRENT COUNT: {}", current);
    info!("LAPIC timer configured");
    }
}

/// Send End-of-Interrupt (EOI) to LAPIC.
/// Must be called after handling an interrupt.
pub fn send_eoi() {
    lapic_write(LapicRegister::EOI, 0);
}

/// Read a Model Specific Register (MSR).
#[inline]
pub fn read_msr(msr: u32) -> u64 {
    let low: u32;
    let high: u32;
    unsafe {
        asm!(
            "rdmsr",
            in("ecx") msr,
            out("eax") low,
            out("edx") high,
        );
    }
    ((high as u64) << 32) | (low as u64)
}

/// Get LAPIC ID via CPUID instruction.
/// Returns the APIC ID from CPUID leaf 1 (bits 24–31 of EBX).
#[inline]
pub fn cpuid_apic_id() -> u8 {
    let apic_id: u32;
    unsafe {
        asm!(
            "cpuid",
            in("eax") 1,
            lateout("edi") apic_id, // APIC ID returned in EBX → mapped to EDI
            out("ecx") _,
            out("edx") _,
            options(nomem, nostack),
        );
    }
    ((apic_id >> 24) & 0xFF) as u8
}


