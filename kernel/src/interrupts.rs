
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};
use lazy_static::lazy_static;
use spin;
use crate::gdt::{DOUBLE_FAULT_IST_INDEX, LAPIC_IST_INDEX};
use crate::{print, println};
use crate::hlt_loop;
use crate::apic::apic::send_eoi;

pub const LAPIC_TIMER_VECTOR: u8 = 0x30;

lazy_static! {
  static ref IDT: InterruptDescriptorTable = {
    let mut idt = InterruptDescriptorTable::new();
    
    idt.divide_error.set_handler_fn(divide_error_handler);
    idt.debug.set_handler_fn(debug_handler);
    idt.non_maskable_interrupt.set_handler_fn(non_maskable_interrupt_handler);
    idt.breakpoint.set_handler_fn(breakpoint_handler);
    idt.overflow.set_handler_fn(overflow_handler);
    idt.bound_range_exceeded.set_handler_fn(bound_range_exceeded_handler);
    idt.invalid_opcode.set_handler_fn(invalid_opcode_handler);
    idt.device_not_available.set_handler_fn(device_not_available_handler);
    idt.page_fault.set_handler_fn(page_fault_handler);
  
    unsafe {
    idt.double_fault
          .set_handler_fn(double_fault_handler)
          .set_stack_index(DOUBLE_FAULT_IST_INDEX);
  };
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
   // idt.vmm_communication_exception.set_handler_fn(vmm_communication_exception_handler);
    idt.security_exception.set_handler_fn(security_exception_handler);

    
    
  unsafe {
    idt[LAPIC_TIMER_VECTOR as usize]
    .set_handler_fn(lapic_timer_handler)
    .set_stack_index(LAPIC_IST_INDEX);
  }

  

    idt

  };


}

pub fn init_idt() { 

  IDT.load();
 
  }






extern "x86-interrupt" fn divide_error_handler(stack_frame: InterruptStackFrame) {
  println!("EXCEPTION: DIVIDE ERROR\n{:#?}", stack_frame);
  hlt_loop();
}

extern "x86-interrupt" fn debug_handler(stack_frame: InterruptStackFrame) {
  println!("EXCEPTION: DEBUG ERROR\n{:#?}", stack_frame);
  hlt_loop();
}

extern "x86-interrupt" fn non_maskable_interrupt_handler(stack_frame: InterruptStackFrame) {
  println!("EXCEPTION: NON MASKABLE ERROR\n{:#?}", stack_frame);
  hlt_loop();
}


extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {

 println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
 

}

extern "x86-interrupt" fn overflow_handler(stack_frame: InterruptStackFrame) {
  println!("EXCEPTION: OVERFLOW HANDLER\n{:#?}", stack_frame);
  hlt_loop();
}

extern "x86-interrupt" fn bound_range_exceeded_handler(stack_frame: InterruptStackFrame) {
  println!("EXCEPTION: BOUND RANGE EXCEEDED HANDLER\n{:#?}", stack_frame);
  hlt_loop();
}

extern "x86-interrupt" fn invalid_opcode_handler(stack_frame: InterruptStackFrame) {
  println!("EXCEPTION: INVALID OPCODE HANDLER\n{:#?}", stack_frame);
  hlt_loop();
  }

extern "x86-interrupt" fn device_not_available_handler(stack_frame: InterruptStackFrame) {
  println!("EXCEPTION: DEVICE NOT AVAILABLE HANDLER\n{:#?}", stack_frame);
  hlt_loop();
}

extern "x86-interrupt" fn page_fault_handler(
  stack_frame: InterruptStackFrame,
  error_code: PageFaultErrorCode,
) {
  use x86_64::registers::control::Cr2;
  println!("EXCEPTION: PAGE FAULT");
  println!("Accessed Address: {:?}", Cr2::read());
  println!("Error Code: {:?}", error_code);
  println!("{:#?}", stack_frame);
  hlt_loop();

}

extern "x86-interrupt" fn double_fault_handler(
  stack_frame: InterruptStackFrame,
  _error_code: u64
) -> ! {
  println!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
  hlt_loop();

}


extern "x86-interrupt" fn invalid_tss_handler(stack_frame: InterruptStackFrame, _error_code: u64) {
  println!("EXCEPTION: INVALID TSS HANDLER\n{:#?}", stack_frame);
  hlt_loop();
}

extern "x86-interrupt" fn segment_not_present_handler(stack_frame: InterruptStackFrame, _error_code: u64) {
  println!("EXCEPTION: SEGMENT NOT PRESENT HANDLER\n{:#?}", stack_frame);
  hlt_loop();}

extern "x86-interrupt" fn stack_segment_fault_handler(stack_frame: InterruptStackFrame, _error_code: u64) {
  println!("EXCEPTION: STACK SEGMENT FAULT HANDLER\n{:#?}", stack_frame); 
  hlt_loop();
}

extern "x86-interrupt" fn general_protection_fault_handler(stack_frame: InterruptStackFrame, _error_code: u64) {
 println!("EXCEPTION: GENERAL PROTECTION FAULT HANDLER\n{:#?}", stack_frame);
 println!("Error Code: {}", _error_code);
  hlt_loop();
}



extern "x86-interrupt" fn x87_floating_point_handler(stack_frame: InterruptStackFrame) {
  println!("EXCEPTION: FLOATING POINT HANDLER\n{:#?}", stack_frame);
  hlt_loop();
}

extern "x86-interrupt" fn alignment_check_handler(stack_frame: InterruptStackFrame, _error_code: u64) {
  println!("EXCEPTION: ALIGNMENT CHECK HANDLER\n{:#?}", stack_frame);
  hlt_loop();
}

extern "x86-interrupt" fn machine_check_handler(stack_frame: InterruptStackFrame) -> ! {
  println!("EXCEPTION: MACHINE CHECK HANDLER\n{:#?}", stack_frame);
  hlt_loop();   
}

extern "x86-interrupt" fn simd_floating_point_handler(stack_frame: InterruptStackFrame) {
  println!("EXCEPTION: SIMD FLOATING POINT HANDLER\n{:#?}", stack_frame);
  hlt_loop();}

extern "x86-interrupt" fn virtualization_handler(stack_frame: InterruptStackFrame) {
  println!("EXCEPTION: VIRTUALIZATION HANDLER\n{:#?}", stack_frame);
  hlt_loop();
}

extern "x86-interrupt" fn cp_protection_exception_handler(stack_frame: InterruptStackFrame, _error_code: u64) {
  println!("EXCEPTION: CP PROTECTION EXCEPTION HANDLER\n{:#?}", stack_frame);
  hlt_loop();
}

extern "x86-interrupt" fn hv_injection_exception_handler(stack_frame: InterruptStackFrame) {
  println!("EXCEPTION: HV INJECTION EXCEPTION HANDLER\n{:#?}", stack_frame);
  hlt_loop();
}

//extern "x86-interrupt" fn vmm_communication_exception_handler(stack_frame: InterruptStackFrame) {
 //  println!("EXCEPTION: VMM COMMUNICATION EXCEPTION HANDLER\n{:#?}", stack_frame);
   //hlt_loop();
//}

extern "x86-interrupt" fn security_exception_handler(stack_frame: InterruptStackFrame, _error_code: u64) {
  println!("EXCEPTION: SECURITY EXCEPTION HANDLER\n{:#?}", stack_frame);
  hlt_loop();
}






extern "x86-interrupt" fn lapic_timer_handler(_stack_frame: InterruptStackFrame) {
    send_eoi();
}


