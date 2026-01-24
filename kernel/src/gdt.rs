use lazy_static::lazy_static;
use x86_64::structures::gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector};
use x86_64::structures::tss::TaskStateSegment;
use x86_64::VirtAddr;
use crate::stack::{STACK, LAPIC_STACK};

/// IST index for the double-fault handler.
pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

/// IST index for LAPIC timer and page-fault handlers.
pub const LAPIC_IST_INDEX: u16 = 1;

/// Size of each dedicated IST stack (bytes).
pub const STACK_SIZE: usize = 128 * 1024;

/// Global Task State Segment with two IST stacks:
/// - IST0: double fault
/// - IST1: LAPIC timer / page faults
lazy_static! {
    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();

        // Double fault stack: use end address as top of stack.
        let df_stack_start = VirtAddr::from_ptr(unsafe { core::ptr::addr_of!(STACK.0) });
        let df_stack_end = df_stack_start + STACK_SIZE;
        // Ensure 16-byte alignment per x86_64 ABI requirements.
        assert_eq!(df_stack_end.as_u64() % 16, 0);
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = df_stack_end;

        // LAPIC/IST stack: used for page fault and LAPIC timer handlers.
        let lapic_stack_start = VirtAddr::from_ptr(unsafe { core::ptr::addr_of!(LAPIC_STACK.0) });
        let lapic_stack_end = lapic_stack_start + STACK_SIZE;
        assert_eq!(lapic_stack_end.as_u64() % 16, 0);
        tss.interrupt_stack_table[LAPIC_IST_INDEX as usize] = lapic_stack_end;

        tss
    };
}

/// Global Descriptor Table and its selectors.
/// Contains kernel code/data segments and the TSS descriptor.
lazy_static! {
    static ref GDT: (GlobalDescriptorTable, Selectors) = {
        let mut gdt = GlobalDescriptorTable::new();

        let code_selector = gdt.add_entry(Descriptor::kernel_code_segment());
        let data_selector = gdt.add_entry(Descriptor::kernel_data_segment());
        let tss_selector  = gdt.add_entry(Descriptor::tss_segment(&TSS));

        // NEW: user-mode segments
        let user_code_selector = gdt.add_entry(Descriptor::user_code_segment());
        let user_data_selector = gdt.add_entry(Descriptor::user_data_segment());

        (
            gdt,
            Selectors {
                code_selector,
                data_selector,
                tss_selector,
                user_code_selector,
                user_data_selector,
            }
        )
    };
}

/// Convenience container for GDT segment selectors.
struct Selectors {
    code_selector: SegmentSelector,
    data_selector: SegmentSelector,
    tss_selector: SegmentSelector,
    user_code_selector: SegmentSelector,
    user_data_selector: SegmentSelector,
}

/// Load the GDT and activate segment registers and TSS.
/// Safety: must be called once during early kernel init.
pub fn init() {
    use x86_64::instructions::segmentation::{CS, DS, ES, SS, Segment};
    use x86_64::instructions::tables::load_tss;

    // Load GDT then set segment registers and TSS.
    GDT.0.load();
    unsafe {
        CS::set_reg(GDT.1.code_selector);
        DS::set_reg(GDT.1.data_selector);
        ES::set_reg(GDT.1.data_selector);
        SS::set_reg(GDT.1.data_selector);
        load_tss(GDT.1.tss_selector);
    }
}

pub fn user_code_selector() -> SegmentSelector {
    GDT.1.user_code_selector
}

pub fn user_data_selector() -> SegmentSelector {
    GDT.1.user_data_selector
}

pub fn kernel_code_selector() -> SegmentSelector {
    GDT.1.code_selector
}

pub fn kernel_data_selector() -> SegmentSelector {
    GDT.1.data_selector
}
