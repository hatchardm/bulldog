use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};
use crate::gdt::{DOUBLE_FAULT_IST_INDEX, LAPIC_IST_INDEX};
use log::{info, error};
use crate::apic::send_eoi;
use core::sync::atomic::{AtomicUsize, AtomicU64};
use crate::time::tick;
use x86_64::instructions::interrupts;
use core::cell::UnsafeCell;
use crate::syscall::SYSCALL_VECTOR;

/// LAPIC timer interrupt vector.
pub const LAPIC_TIMER_VECTOR: u8 = 0x31;



/// Spurious interrupt vector (used to enable LAPIC).
const SPURIOUS_VECTOR: u8 = 0xFF;

/// Tracks LAPIC timer hits (atomic counter).
pub static LAPIC_HITS: AtomicUsize = AtomicUsize::new(0);

/// Raw LAPIC state (unsafe globals for debugging).
pub static mut LAPIC_RSP: u64 = 0;
pub static mut LAPIC_HITS_RAW: u64 = 0;

/// A globally allocated IDT with interior mutability and explicit Sync.
/// We guarantee safe mutation by only writing with interrupts disabled.
struct IdtCell(UnsafeCell<InterruptDescriptorTable>);
unsafe impl Sync for IdtCell {}

static IDT: IdtCell = IdtCell(UnsafeCell::new(InterruptDescriptorTable::new()));

#[inline]
fn idt_ref() -> &'static InterruptDescriptorTable {
    unsafe { &*IDT.0.get() }
}

#[inline]
pub fn idt_mut() -> &'static mut InterruptDescriptorTable {
    unsafe { &mut *IDT.0.get() }
}

/// Initialize and load the IDT.
/// Logs handler addresses for selected vectors.
pub fn init_idt() {
    // Mutate the global IDT in a safe window with interrupts disabled.
    interrupts::without_interrupts(|| {
        let idt = idt_mut();

        // Core exceptions
        idt.divide_error.set_handler_fn(divide_error_handler);
        idt.debug.set_handler_fn(debug_handler);
        idt.non_maskable_interrupt.set_handler_fn(non_maskable_interrupt_handler);
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt.overflow.set_handler_fn(overflow_handler);
        idt.bound_range_exceeded.set_handler_fn(bound_range_exceeded_handler);
        idt.invalid_opcode.set_handler_fn(invalid_opcode_handler);
        idt.device_not_available.set_handler_fn(device_not_available_handler);
        idt.invalid_tss.set_handler_fn(invalid_tss_handler);
        idt.segment_not_present.set_handler_fn(segment_not_present_handler);
        idt.stack_segment_fault.set_handler_fn(stack_segment_fault_handler);
        idt.general_protection_fault.set_handler_fn(general_protection_fault_handler);
        idt.x87_floating_point.set_handler_fn(x87_floating_point_handler);
        idt.alignment_check.set_handler_fn(alignment_check_handler);
        idt.machine_check.set_handler_fn(machine_check_handler);
        idt.simd_floating_point.set_handler_fn(simd_floating_point_handler);
        idt.virtualization.set_handler_fn(virtualization_handler);
        idt.cp_protection_exception.set_handler_fn(cp_protection_exception_handler);
        idt.hv_injection_exception.set_handler_fn(hv_injection_exception_handler);
        idt.security_exception.set_handler_fn(security_exception_handler);

        // IST exceptions (use alternate stacks for reliability).
        unsafe {
            idt.page_fault
                .set_handler_fn(page_fault_handler)
                .set_stack_index(LAPIC_IST_INDEX);

            idt.double_fault
                .set_handler_fn(double_fault_handler)
                .set_stack_index(DOUBLE_FAULT_IST_INDEX);

            idt[LAPIC_TIMER_VECTOR as usize]
                .set_handler_fn(lapic_timer_handler)
                .set_stack_index(LAPIC_IST_INDEX);

            idt[SPURIOUS_VECTOR as usize].set_handler_fn(spurious_handler);
        }

        // Example custom vectors
        unsafe {
            idt[32].set_handler_fn(log_vector_32);
            idt[33].set_handler_fn(log_vector_33);
            idt[48].set_handler_fn(unhandled_vector_48);
            idt[50].set_handler_fn(log_vector_50);
            idt[255].set_handler_fn(unhandled_vector_255);
        }

        // Fallback handlers for unassigned vectors
        for i in 0..256 {
            let skip = i == 8
                || (10..=15).contains(&i)
                || (17..=18).contains(&i)
                || (21..=27).contains(&i)
                || (29..=31).contains(&i)
                || i == LAPIC_TIMER_VECTOR as usize;
                || i == SYSCALL_VECTOR as usize; // <-- skip syscall vector

            if skip || idt[i].handler_addr().as_u64() != 0 {
                continue;
            }

            unsafe {
                idt[i].set_handler_fn(default_handler);
            }
        }

        // Log selected vectors after registration
        for i in 48..=50 {
            let addr = idt[i].handler_addr().as_u64();
            if addr == 0 {
                error!("IDT[{}] is NOT set", i);
            } else {
                info!("IDT[{}] handler address: {:#x}", i, addr);
            }
        }

        // Load the IDT (executes lidt) on the 'static reference
        unsafe { idt_ref().load(); }
    });
}


// === Exception Handlers ===

extern "x86-interrupt" fn divide_error_handler(stack_frame: InterruptStackFrame) {
    error!("EXCEPTION: DIVIDE ERROR\n{:#?}", stack_frame);
    panic!("EXCEPTION: DIVIDE ERROR");
}

extern "x86-interrupt" fn debug_handler(stack_frame: InterruptStackFrame) {
    error!("EXCEPTION: DEBUG\n{:#?}", stack_frame);
    panic!("EXCEPTION: DEBUG");
}

extern "x86-interrupt" fn non_maskable_interrupt_handler(stack_frame: InterruptStackFrame) {
    error!("EXCEPTION: NON MASKABLE\n{:#?}", stack_frame);
    panic!("EXCEPTION: NON MASKABLE");
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    error!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn overflow_handler(stack_frame: InterruptStackFrame) {
    error!("EXCEPTION: OVERFLOW\n{:#?}", stack_frame);
    panic!("EXCEPTION: OVERFLOW");
}

extern "x86-interrupt" fn bound_range_exceeded_handler(stack_frame: InterruptStackFrame) {
    error!("EXCEPTION: BOUND RANGE EXCEEDED\n{:#?}", stack_frame);
    panic!("EXCEPTION: BOUND RANGE EXCEEDED");
}

extern "x86-interrupt" fn invalid_opcode_handler(stack_frame: InterruptStackFrame) {
    error!("EXCEPTION: INVALID OPCODE\n{:#?}", stack_frame);
    panic!("Invalid opcode");
}

extern "x86-interrupt" fn device_not_available_handler(stack_frame: InterruptStackFrame) {
    error!("EXCEPTION: DEVICE NOT AVAILABLE\n{:#?}", stack_frame);
    panic!("EXCEPTION: DEVICE NOT AVAILABLE");
}

extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    use x86_64::registers::control::Cr2;
    error!("EXCEPTION: PAGE FAULT");
    error!("Accessed Address: {:?}", Cr2::read());
    error!("Error Code: {:?}", error_code);
    error!("{:#?}", stack_frame);
    panic!("EXCEPTION: PAGE FAULT");
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    error!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
    panic!("EXCEPTION: DOUBLE FAULT");
}

extern "x86-interrupt" fn invalid_tss_handler(stack_frame: InterruptStackFrame, _error_code: u64) {
    error!("EXCEPTION: INVALID TSS\n{:#?}", stack_frame);
    panic!("EXCEPTION: INVALID TSS");
}

extern "x86-interrupt" fn segment_not_present_handler(stack_frame: InterruptStackFrame, _error_code: u64) {
    error!("EXCEPTION: SEGMENT NOT PRESENT\n{:#?}", stack_frame);
    panic!("EXCEPTION: SEGMENT NOT PRESENT");
}

extern "x86-interrupt" fn stack_segment_fault_handler(stack_frame: InterruptStackFrame, _error_code: u64) {
    error!("EXCEPTION: STACK SEGMENT FAULT\n{:#?}", stack_frame);
    panic!("EXCEPTION: STACK SEGMENT FAULT");
}

extern "x86-interrupt" fn general_protection_fault_handler(stack_frame: InterruptStackFrame, _error_code: u64) {
    error!("EXCEPTION: GENERAL PROTECTION FAULT\n{:#?}", stack_frame);
    error!("Error Code: {}", _error_code);
    panic!("EXCEPTION: GENERAL PROTECTION FAULT");
}

extern "x86-interrupt" fn x87_floating_point_handler(stack_frame: InterruptStackFrame) {
    error!("EXCEPTION: X87 FLOATING POINT\n{:#?}", stack_frame);
    panic!("EXCEPTION: X87 FLOATING POINT");
}

extern "x86-interrupt" fn alignment_check_handler(stack_frame: InterruptStackFrame, _error_code: u64) {
    error!("EXCEPTION: ALIGNMENT CHECK\n{:#?}", stack_frame);
    panic!("EXCEPTION: ALIGNMENT CHECK");
}

extern "x86-interrupt" fn machine_check_handler(stack_frame: InterruptStackFrame) -> ! {
    error!("EXCEPTION: MACHINE CHECK\n{:#?}", stack_frame);
    panic!("EXCEPTION: MACHINE CHECK");
}

extern "x86-interrupt" fn simd_floating_point_handler(stack_frame: InterruptStackFrame) {
    error!("EXCEPTION: SIMD FLOATING POINT\n{:#?}", stack_frame);
    panic!("EXCEPTION: SIMD FLOATING POINT");
}

extern "x86-interrupt" fn virtualization_handler(stack_frame: InterruptStackFrame) {
    error!("EXCEPTION: VIRTUALIZATION\n{:#?}", stack_frame);
    panic!("EXCEPTION: VIRTUALIZATION");
}

extern "x86-interrupt" fn cp_protection_exception_handler(stack_frame: InterruptStackFrame, _error_code: u64) {
    error!("EXCEPTION: CP PROTECTION\n{:#?}", stack_frame);
    panic!("EXCEPTION: CP PROTECTION");
}

extern "x86-interrupt" fn hv_injection_exception_handler(stack_frame: InterruptStackFrame) {
    error!("EXCEPTION: HV INJECTION\n{:#?}", stack_frame);
    panic!("EXCEPTION: HV INJECTION");
}

extern "x86-interrupt" fn security_exception_handler(stack_frame: InterruptStackFrame, _error_code: u64) {
    error!("EXCEPTION: SECURITY\n{:#?}", stack_frame);
    panic!("EXCEPTION: SECURITY");
}

/// LAPIC timer interrupt handler.
/// Increments kernel tick and sends EOI to LAPIC.
extern "x86-interrupt" fn lapic_timer_handler(_stack_frame: InterruptStackFrame) {
    tick();
    send_eoi();
}

/// Spurious interrupt handler.
/// Logs and acknowledges the interrupt.
extern "x86-interrupt" fn spurious_handler(_stack_frame: InterruptStackFrame) {
    error!("SPURIOUS INTERRUPT");
    send_eoi();
}

/// Default handler for unassigned vectors.
extern "x86-interrupt" fn default_handler(_stack_frame: InterruptStackFrame) {
    error!("UNHANDLED INTERRUPT");
}

/// Example custom vector handlers.
extern "x86-interrupt" fn log_vector_32(_stack_frame: InterruptStackFrame) {
    error!("UNHANDLED INTERRUPT: vector 32");
}

extern "x86-interrupt" fn log_vector_33(_stack_frame: InterruptStackFrame) {
    error!("UNHANDLED INTERRUPT: vector 33");
}

extern "x86-interrupt" fn unhandled_vector_48(_stack_frame: InterruptStackFrame) {
    error!("UNHANDLED INTERRUPT: vector 48");
}

// Example placeholder for vector 49 if needed.
// extern "x86-interrupt" fn log_vector_49(_stack_frame: InterruptStackFrame) {
//     error!("UNHANDLED INTERRUPT: vector 49");
// }

extern "x86-interrupt" fn log_vector_50(_stack_frame: InterruptStackFrame) {
    error!("UNHANDLED INTERRUPT: vector 50");
}

extern "x86-interrupt" fn unhandled_vector_255(_stack_frame: InterruptStackFrame) {
    error!("UNHANDLED INTERRUPT: vector 255");
}




