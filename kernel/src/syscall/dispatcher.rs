// File: kernel/src/syscall/dispatcher.rs

use log::{info, warn};
use x86_64::structures::idt::InterruptStackFrame;
use x86_64::PrivilegeLevel;
use crate::syscall::table::lookup;
use crate::syscall::errno::{err, errno};
use core::arch::naked_asm;

pub const SYSCALL_VECTOR: u8 = 0x80;

#[unsafe(naked)]
pub extern "C" fn syscall_handler() {
    unsafe {
        naked_asm!(
            r#"
            push rbx
            push rbp
            push r12
            push r13

            mov rbx, rax
            mov rbp, rdi
            mov r12, rsi
            mov r13, rdx

            mov rdi, rbx
            mov rsi, rbp
            mov rdx, r12
            mov rcx, r13

            call rust_dispatch

            pop r13
            pop r12
            pop rbp
            pop rbx

            iretq
            "#
        );
    }
}

#[unsafe(no_mangle)]
extern "C" fn rust_dispatch(num: u64, a0: u64, a1: u64, a2: u64) -> u64 {
    #[cfg(feature = "syscall_tests")]
    info!(
        "dispatch called with num={} a0={:#x} a1={:#x} a2={:#x}",
        num, a0 as usize, a1 as usize, a2 as usize
    );

    let ret = dispatch(num, a0, a1, a2);
    #[cfg(feature = "syscall_tests")]
    info!("syscall num={} ret={}", num, ret);
    ret
}

pub fn dispatch(num: u64, arg0: u64, arg1: u64, arg2: u64) -> u64 {
    match lookup(num) {
        Some(fun) => fun(arg0, arg1, arg2),
        None => {
            warn!("Unknown syscall {}", num);
            err(errno::ENOSYS) // Function not implemented
        }
    }
}

pub fn init_syscall() {
    let mut idt = crate::interrupts::idt_mut();
    unsafe {
        idt[SYSCALL_VECTOR as usize]
            .set_handler_fn(core::mem::transmute::<extern "C" fn(), extern "x86-interrupt" fn(InterruptStackFrame)>(syscall_handler))
            .set_privilege_level(PrivilegeLevel::Ring3);
    }
    info!(
        "Syscall handler initialized at vector {:#x} (DPL=3)",
        SYSCALL_VECTOR
    );
}




