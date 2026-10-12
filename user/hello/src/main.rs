#![no_std]
#![no_main]

use core::arch::naked_asm;
use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[unsafe(link_section = ".rodata")]
static MSG: [u8; 22] = *b"Hello from User Space\n";

#[unsafe(naked)]
#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    naked_asm!(
        "mov rbp, rsp",
        "mov rax, 1",           // SYS_WRITE
        "mov rdi, 1",           // fd = 1
        "lea rsi, [rip + {msg}]",
        "mov rdx, 22",          // len
        "int 0x80",
        "mov rax, 2",           // SYS_EXIT
        "xor rdi, rdi",
        "int 0x80",
        "2: jmp 2b",
        msg = sym MSG,
    );
}