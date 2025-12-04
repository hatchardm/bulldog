//! src/syscall.rs
//! Syscall scaffold for Bulldog (APIC baseline)
//! - Registers an IDT trap at vector 0x80
//! - Kernel-side handler dispatches to syscall stubs
//! - User-side entry uses `int 0x80` for now (no privilege switch yet)

use core::arch::asm;
use log::{info, warn};
use x86_64::structures::idt::InterruptStackFrame;
use core::slice;

pub const SYSCALL_VECTOR: u8 = 0x80;

// Syscall numbers (expand as needed)
pub const SYS_WRITE: u64 = 1;
pub const SYS_EXIT:  u64 = 2;

/// Kernel-side syscall handler.
/// For now, just logs and calls the dispatcher with dummy args.
pub extern "x86-interrupt" fn syscall_handler(_stack_frame: InterruptStackFrame) {
    // Read rax (num), rdi/rsi/rdx (args) and rax as return value
    let (num, a0, a1, a2): (u64, u64, u64, u64);
    unsafe {
        core::arch::asm!(
            "/* read rax,rdi,rsi,rdx into temp regs */",
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

    // Place return value back into rax so user-side gets it
    unsafe {
        core::arch::asm!(
            "mov rax, {ret}",
            ret = in(reg) ret,
            options(nostack, preserves_flags)
        );
    }

    // Optional: log once per syscall to avoid noisy output
    info!("syscall num={} ret={}", num, ret);
}


/// Initialization hook to register the syscall handler.
/// Call this once during kernel_init, before interrupts are enabled.
pub fn init_syscall() {
    #[cfg(feature = "idt_helpers")]
    {
        crate::interrupts::register_trap(SYSCALL_VECTOR, syscall_handler);
        info!("Syscall handler registered at vector {:#x} via register_trap", SYSCALL_VECTOR);
        return;
    }

    let mut idt = crate::interrupts::idt_mut();
    idt[SYSCALL_VECTOR as usize].set_handler_fn(syscall_handler);
    info!("Syscall handler initialized at vector {:#x}", SYSCALL_VECTOR);
}

/// Dispatcher: routes syscall numbers to stub implementations.
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

/// Stub: write to framebuffer/logger
fn sys_write(fd: u64, buf_ptr: u64, len: u64) -> u64 {
    use core::slice;
    info!("sys_write(fd={}, ptr=0x{:x}, len={})", fd, buf_ptr, len);

    unsafe {
        let buf = slice::from_raw_parts(buf_ptr as *const u8, len as usize);
        if let Ok(s) = core::str::from_utf8(buf) {
            info!("Echo from user: {}", s);
        }
    }
    0
}


/// Stub: exit process
fn sys_exit(code: u64) -> u64 {
    info!("sys_exit(code={})", code);
    // TODO: mark process as terminated
    0
}

/// User-side entry trigger (currently from kernel context; later from ring 3).
#[inline(always)]
pub unsafe fn syscall_entry() {
    asm!(
        "int 0x80",
        options(nostack, nomem)
    );
}


