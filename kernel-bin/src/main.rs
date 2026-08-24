#![no_std]
#![no_main]

use boot_proto::MemoryRegionKind;
use boot_proto::BootInfo;

use kernel::paging;
use kernel::serial::{serial_println, serial_print, serial_print_hex_u64};
use kernel::kernel_main;   // FIXED: import from kernel crate

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {
        unsafe { core::arch::asm!("hlt"); }
    }
}






#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.entry_point")]
pub fn kernel_entry(boot_info: &mut BootInfo) -> ! {
    // --- Entry diagnostics (keep these) ---
    let rsp: u64;
    unsafe { core::arch::asm!("mov {}, rsp", out(reg) rsp); }

    serial_println("RSP at entry:");
    serial_print_hex_u64(rsp);
    serial_println("");

    serial_println("=== BOOTINFO CHECK AT KERNEL ENTRY ===");

    serial_println("boot_info.framebuffer.addr:");
    serial_print_hex_u64(boot_info.framebuffer.addr);
    serial_println("");

    serial_println("boot_info.framebuffer_virt:");
    serial_print_hex_u64(boot_info.framebuffer_virt);
    serial_println("");

    serial_println("boot_info.physical_memory_offset:");
    serial_print_hex_u64(boot_info.physical_memory_offset);
    serial_println("");

    // --- Paging ---
    serial_println("=== ENTERED paging::init ===");
    unsafe { paging::init(boot_info); }

    // --- Final framebuffer mapping result (keep this) ---
    serial_println("framebuffer mapped at:");
    serial_print_hex_u64(boot_info.framebuffer_virt);
    serial_println("");

    // --- Continue into kernel ---
    kernel_main(boot_info);
}




























