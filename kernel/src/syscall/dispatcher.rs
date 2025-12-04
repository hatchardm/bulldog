use log::{info, warn};
use x86_64::structures::idt::InterruptStackFrame;
use core::arch::asm;

use crate::syscall::stubs::{SYS_WRITE, SYS_EXIT, sys_write, sys_exit};

pub const SYSCALL_VECTOR: u8 = 0x80;

/// Kernel-side syscall handler.
pub extern "x86-interrupt" fn syscall_handler(_stack_frame: InterruptStackFrame) {
    let (num, a0, a1, a2): (u64, u64, u64, u64);
    unsafe {
        asm!(
            "mov {num}, rax",
            "mov {a0}, rdi",
            "mov {a1}, rsi",
            "mov {a2}, rdx",
            num = lateout(reg) num,
            a0  = lateout(reg) a0,
            a1  = lateout(reg) a1,
            a2  = lateout(reg) a2,
            options(nostack, preserves_flags)
        );
    }

    let ret = dispatch(num, a0, a1, a2);

    unsafe {
        asm!(
            "mov rax, {ret}",
            ret = in(reg) ret,
            options(nostack, preserves_flags)
        );
    }

    info!("syscall num={} ret={}", num, ret);
}

/// Initialization hook
pub fn init_syscall() {
    let mut idt = crate::interrupts::idt_mut();
    idt[SYSCALL_VECTOR as usize].set_handler_fn(syscall_handler);
    info!("Syscall handler initialized at vector {:#x}", SYSCALL_VECTOR);
}

/// Dispatcher
pub fn dispatch(num: u64, arg0: u64, arg1: u64, arg2: u64) -> u64 {
    match num {
        SYS_WRITE => sys_write(arg0, arg1, arg2),
        SYS_EXIT  => sys_exit(arg0),
        _ => {
            warn!("Unknown syscall {}", num);
            u64::MAX
        }
    }
}

/// User-side entry trigger
#[inline(always)]
pub unsafe fn syscall_entry() {
    asm!("int 0x80", options(nostack, nomem));
}
