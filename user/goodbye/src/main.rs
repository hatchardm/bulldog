#![no_std]
#![no_main]

use core::arch::asm;
use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

fn sys_write(fd: u64, buf: *const u8, len: u64) -> i64 {
    let mut ret: u64;
    unsafe {
        asm!(
            "int 0x80",
            inlateout("rax") 1u64 => ret,
            in("rdi") fd,
            in("rsi") buf,
            in("rdx") len,
            options(nostack, preserves_flags)
        );
    }
    ret as i64
}

fn sys_exit(code: u64) -> ! {
    unsafe {
        asm!(
            "int 0x80",
            in("rax") 2u64,
            in("rdi") code,
            options(noreturn, nostack)
        );
    }
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