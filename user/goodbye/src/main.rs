#![no_std]
#![no_main]

use core::arch::asm;
use core::panic::PanicInfo;
use user_lib::{sys_write, sys_exit};

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[unsafe(link_section = ".rodata")]
static MSG: [u8; 24] = *b"Goodbye from User Space\n";

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    let msg_ptr: *const u8;
    unsafe {
        asm!(
            "lea {}, [rip + {}]",
            out(reg) msg_ptr,
            sym MSG,
            options(nostack, preserves_flags)
        );
    }
    let _ = sys_write(1, msg_ptr, MSG.len() as u64);
    sys_exit(0);
}
