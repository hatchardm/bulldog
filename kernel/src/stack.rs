use crate::gdt::STACK_SIZE;
use x86_64::VirtAddr;

/// A 16‑byte aligned stack used for the double fault IST.
/// Alignment is required by the x86_64 ABI for stack operations.
#[repr(align(16))]
pub struct AlignedStack(pub [u8; STACK_SIZE]);

/// Global kernel stack for double fault handling.
/// Placed in the `.stack` section and exported with `no_mangle`
/// so the linker and TSS setup can reference it directly.
#[unsafe(link_section = ".stack")]
#[unsafe(no_mangle)]
pub static mut STACK: AlignedStack = AlignedStack([0; STACK_SIZE]);

/// Return the starting virtual address of the global kernel stack.
/// Used when configuring the TSS IST entry.
pub fn get_stack_start() -> VirtAddr {
    unsafe { VirtAddr::from_ptr(core::ptr::addr_of!(STACK.0)) }
}

/// A 16‑byte aligned stack used for LAPIC timer and page fault IST.
/// Separate from the double fault stack to isolate failure contexts.
#[repr(align(16))]
pub struct Stack(pub [u8; STACK_SIZE]);

/// Global LAPIC IST stack.
/// Used for LAPIC timer and page fault handlers to ensure
/// reliable execution even if the main kernel stack is corrupted.
pub static LAPIC_STACK: Stack = Stack([0; STACK_SIZE]);

