use crate::gdt::STACK_SIZE;

#[repr(align(16))]
pub struct AlignedStack(pub [u8; STACK_SIZE]);


#[unsafe(link_section = ".stack")]
#[unsafe(no_mangle)]
pub static mut STACK: AlignedStack = AlignedStack([0; STACK_SIZE]);


use x86_64::VirtAddr;

pub fn get_stack_start() -> VirtAddr {
    unsafe { VirtAddr::from_ptr(core::ptr::addr_of!(STACK.0)) }
}

#[repr(align(16))]
pub struct Stack(pub [u8; STACK_SIZE]);

pub static LAPIC_STACK: Stack = Stack([0; STACK_SIZE]);
