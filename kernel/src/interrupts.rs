use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};
use lazy_static::lazy_static;
use spin;
use crate::gdt::{DOUBLE_FAULT_IST_INDEX, LAPIC_IST_INDEX};
//use crate::{print, println};
use crate::hlt_loop;
use crate::apic::send_eoi;
use core::sync::atomic::{AtomicUsize, AtomicU64, Ordering};
use crate::time::tick;



pub const LAPIC_TIMER_VECTOR: u8 = 0x31;
const SPURIOUS_VECTOR: u8 = 0xFF;
pub static LAPIC_HITS: AtomicUsize = AtomicUsize::new(0);
pub static mut LAPIC_RSP: u64 = 0;
pub static mut LAPIC_HITS_RAW: u64 = 0;

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();

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

        // IST exceptions
        unsafe {
            idt.page_fault
                .set_handler_fn(page_fault_handler)
                .set_stack_index(LAPIC_IST_INDEX);

            idt.double_fault
                .set_handler_fn(double_fault_handler)
                .set_stack_index(DOUBLE_FAULT_IST_INDEX);

            idt[LAPIC_TIMER_VECTOR as usize]
                .set_handler_fn(lapic_timer_handler);
             //   .set_stack_index(LAPIC_IST_INDEX);

            idt[SPURIOUS_VECTOR as usize].set_handler_fn(spurious_handler);
        }

        unsafe {
       idt[32].set_handler_fn(log_vector_32);
       idt[33].set_handler_fn(log_vector_33);
       ;




       idt[48].set_handler_fn(unhandled_vector_48); // 0x30
       idt[255].set_handler_fn(unhandled_vector_255); // 0xFF

     //  idt[49].set_handler_fn(log_vector_49);
       idt[50].set_handler_fn(log_vector_50);
    
        }



        // Fallback handlers for unassigned vectors
        for i in 0..256 {
            let skip = i == 8
            || (10..=15).contains(&i)
            || (17..=18).contains(&i)
            || (21..=27).contains(&i)
            || (29..=31).contains(&i)
            || i == LAPIC_TIMER_VECTOR as usize;


            if skip || idt[i].handler_addr().as_u64() != 0 {
                continue;
            }

            unsafe {
                idt[i].set_handler_fn(default_handler);
            }
        }

        idt
    };
}

pub fn init_idt() {
    // Inspect handler addresses for vectors 48–50
    for i in 48..=50 {
        let addr = IDT[i].handler_addr().as_u64();
        if addr == 0 {
         //   println!("IDT[{}] is NOT set", i);
        } else {
          //  println!("IDT[{}] handler address: {:#x}", i, addr);
        }
    }

    IDT.load();
   // println!("IDT loaded");
}


// === Exception Handlers ===

extern "x86-interrupt" fn divide_error_handler(stack_frame: InterruptStackFrame) {
  //  println!("EXCEPTION: DIVIDE ERROR\n{:#?}", stack_frame);
    hlt_loop();
}

extern "x86-interrupt" fn debug_handler(stack_frame: InterruptStackFrame) {
  //  println!("EXCEPTION: DEBUG\n{:#?}", stack_frame);
    hlt_loop();
}

extern "x86-interrupt" fn non_maskable_interrupt_handler(stack_frame: InterruptStackFrame) {
  //  println!("EXCEPTION: NON MASKABLE\n{:#?}", stack_frame);
    hlt_loop();
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
  //  println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn overflow_handler(stack_frame: InterruptStackFrame) {
  //  println!("EXCEPTION: OVERFLOW\n{:#?}", stack_frame);
    hlt_loop();
}

extern "x86-interrupt" fn bound_range_exceeded_handler(stack_frame: InterruptStackFrame) {
   // println!("EXCEPTION: BOUND RANGE EXCEEDED\n{:#?}", stack_frame);
    hlt_loop();
}

extern "x86-interrupt" fn invalid_opcode_handler(stack_frame: InterruptStackFrame) {
 //  println!("EXCEPTION: INVALID OPCODE\n{:#?}", stack_frame);
    hlt_loop();
}

extern "x86-interrupt" fn device_not_available_handler(stack_frame: InterruptStackFrame) {
  //  println!("EXCEPTION: DEVICE NOT AVAILABLE\n{:#?}", stack_frame);
    hlt_loop();
}

extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    use x86_64::registers::control::Cr2;
  //  println!("EXCEPTION: PAGE FAULT");
  //  println!("Accessed Address: {:?}", Cr2::read());
 //   println!("Error Code: {:?}", error_code);
 // println!("{:#?}", stack_frame);
    hlt_loop();
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
 //   println!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
    hlt_loop();
}

extern "x86-interrupt" fn invalid_tss_handler(stack_frame: InterruptStackFrame, _error_code: u64) {
  //  println!("EXCEPTION: INVALID TSS\n{:#?}", stack_frame);
    hlt_loop();
}

extern "x86-interrupt" fn segment_not_present_handler(stack_frame: InterruptStackFrame, _error_code: u64) {
 //   println!("EXCEPTION: SEGMENT NOT PRESENT\n{:#?}", stack_frame);
    hlt_loop();
}

extern "x86-interrupt" fn stack_segment_fault_handler(stack_frame: InterruptStackFrame, _error_code: u64) {
 //   println!("EXCEPTION: STACK SEGMENT FAULT\n{:#?}", stack_frame);
    hlt_loop();
}

extern "x86-interrupt" fn general_protection_fault_handler(stack_frame: InterruptStackFrame, _error_code: u64) {
 //   println!("EXCEPTION: GENERAL PROTECTION FAULT\n{:#?}", stack_frame);
  //  println!("Error Code: {}", _error_code);
    hlt_loop();
}

extern "x86-interrupt" fn x87_floating_point_handler(stack_frame: InterruptStackFrame) {
  //  println!("EXCEPTION: X87 FLOATING POINT\n{:#?}", stack_frame);
    hlt_loop();
}

extern "x86-interrupt" fn alignment_check_handler(stack_frame: InterruptStackFrame, _error_code: u64) {
  //  println!("EXCEPTION: ALIGNMENT CHECK\n{:#?}", stack_frame);
    hlt_loop();
}

extern "x86-interrupt" fn machine_check_handler(stack_frame: InterruptStackFrame) -> ! {
   // println!("EXCEPTION: MACHINE CHECK\n{:#?}", stack_frame);
    hlt_loop();
}

extern "x86-interrupt" fn simd_floating_point_handler(stack_frame: InterruptStackFrame) {
   // println!("EXCEPTION: SIMD FLOATING POINT\n{:#?}", stack_frame);
    hlt_loop();
}

extern "x86-interrupt" fn virtualization_handler(stack_frame: InterruptStackFrame) {
 //   println!("EXCEPTION: VIRTUALIZATION\n{:#?}", stack_frame);
    hlt_loop();
}

extern "x86-interrupt" fn cp_protection_exception_handler(stack_frame: InterruptStackFrame, _error_code: u64) {
  // println!("EXCEPTION: CP PROTECTION\n{:#?}", stack_frame);
    hlt_loop();
}

extern "x86-interrupt" fn hv_injection_exception_handler(stack_frame: InterruptStackFrame) {
   // println!("EXCEPTION: HV INJECTION\n{:#?}", stack_frame);
    hlt_loop();
}

extern "x86-interrupt" fn security_exception_handler(stack_frame: InterruptStackFrame, _error_code: u64) {
   // println!("EXCEPTION: SECURITY\n{:#?}", stack_frame);
    hlt_loop();
}




extern "x86-interrupt" fn lapic_timer_handler(_stack_frame: InterruptStackFrame) {
    let rsp: u64;
    unsafe { core::arch::asm!("mov {}, rsp", out(reg) rsp) };
    unsafe { LAPIC_RSP = rsp };
}





extern "x86-interrupt" fn spurious_handler(_stack_frame: InterruptStackFrame) {
   // println!("SPURIOUS INTERRUPT");
    send_eoi();
}

extern "x86-interrupt" fn default_handler(_stack_frame: InterruptStackFrame) {
  //  println!("UNHANDLED INTERRUPT");
}


extern "x86-interrupt" fn log_vector_32(_stack_frame: InterruptStackFrame) {
 //  println!("UNHANDLED INTERRUPT: vector 32");
}
extern "x86-interrupt" fn log_vector_33(_stack_frame: InterruptStackFrame) {
  //  println!("UNHANDLED INTERRUPT: vector 33");
}

extern "x86-interrupt" fn unhandled_vector_48(_stack_frame: InterruptStackFrame) {
 //   println!("UNHANDLED INTERRUPT: vector 48");
}

//extern "x86-interrupt" fn log_vector_49(_stack_frame: InterruptStackFrame) {
 //   println!("UNHANDLED INTERRUPT: vector 49");
//}


extern "x86-interrupt" fn unhandled_vector_255(_stack_frame: InterruptStackFrame) {
   // println!("UNHANDLED INTERRUPT: vector 255");
}



extern "x86-interrupt" fn log_vector_50(_stack_frame: InterruptStackFrame) {
  //  println!("UNHANDLED INTERRUPT: vector 50");
}



