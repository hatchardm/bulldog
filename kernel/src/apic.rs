
    use core::ptr::write_volatile;
    use crate::interrupts::LAPIC_TIMER_VECTOR;
   // use crate::{print, println};
    use core::ptr::read_volatile;
    use core::arch::asm;


    pub const LAPIC_VIRT_BASE: u64 = 0xFFFF_FF00_0000_0000;
    pub const LAPIC_BASE: usize = LAPIC_VIRT_BASE as usize;

    #[repr(u32)]
    pub enum LapicTimer {
    OneShot = 0b00,
    Periodic = 0b01,
    TscDeadline = 0b10,
}



    #[repr(usize)]
    pub enum LapicRegister {
    LVT_TIMER = 0x320,       // Timer Mode + Vector
    DIVIDE_CONFIG = 0x3E0,
    INITIAL_COUNT = 0x380,
    CURRENT_COUNT = 0x390,
    EOI = 0xB0,
    SVR = 0xF0,
    ID = 0x20,
    VERSION = 0x30,
}





pub fn lapic_read(reg: LapicRegister) -> u32 {
    unsafe {
        let reg_ptr = (LAPIC_VIRT_BASE + reg as u64) as *mut u32;
        read_volatile(reg_ptr)
    }
}


  pub fn lapic_write(reg: LapicRegister, value: u32) {
    unsafe {
        let reg_ptr = (LAPIC_VIRT_BASE + reg as u64) as *mut u32;
    //    println!("lapic_write → VIRT {:#x}", reg_ptr as usize);
        core::ptr::write_volatile(reg_ptr, value);
    }
}



pub fn setup_apic() {
   // println!("Entered setup_apic()");

    // Read LAPIC base MSR
    let apic_base = read_msr(0x1B);
    let base_phys = apic_base & 0xFFFFF000;
    let enabled = (apic_base >> 11) & 1;
   // println!("APIC base MSR: {:#x}", apic_base);
    //println!("APIC physical base: {:#x}", base_phys);
   // println!("LAPIC enabled: {}", enabled);
    if enabled == 0 {
        panic!("LAPIC is not enabled!");
    }

    // LAPIC version
    let version = lapic_read(LapicRegister::VERSION);
   // println!("LAPIC VERSION: {:#x}", version);

    // LAPIC ID (once)
    let id = lapic_read(LapicRegister::ID);
    let cpuid_id = cpuid_apic_id();
  //  println!("LAPIC ID: {:#x}, CPUID APIC ID: {:#x}", id, cpuid_id);

    // Spurious Interrupt Vector Register (SVR)
    lapic_write(LapicRegister::SVR, 0x100 | LAPIC_TIMER_VECTOR as u32);
   // println!("SVR written");

    // Timer setup
    lapic_write(LapicRegister::DIVIDE_CONFIG, 0b0011); // Divide by 16

    let value = LAPIC_TIMER_VECTOR as u32; // One-shot mode

    lapic_write(LapicRegister::LVT_TIMER, value);


    

    let lvt = lapic_read(LapicRegister::LVT_TIMER);
   // println!("LVT_TIMER: {:#x}", lvt);

    lapic_write(LapicRegister::INITIAL_COUNT, 1_000_000); // Adjust as needed

    let current = lapic_read(LapicRegister::CURRENT_COUNT);
  //  println!("LAPIC CURRENT COUNT: {}", current);


   // println!("LAPIC timer configured");
}






    pub fn send_eoi() {
    
        lapic_write(LapicRegister::EOI, 0);

    }


    #[inline]
    pub fn read_msr(msr: u32) -> u64 {
    let low: u32;
    let high: u32;
    unsafe {
        core::arch::asm!(
            "rdmsr",
            in("ecx") msr,
            out("eax") low,
            out("edx") high,
        );
    }
    ((high as u64) << 32) | (low as u64)
    }


#[inline]
pub fn cpuid_apic_id() -> u8 {
    let apic_id: u32;
    unsafe {
        core::arch::asm!(
            "cpuid",
            in("eax") 1,
            lateout("edi") apic_id, // use edi instead of ebx
            out("ecx") _,
            out("edx") _,
            options(nomem, nostack),
        );
    }
    ((apic_id >> 24) & 0xFF) as u8
}

