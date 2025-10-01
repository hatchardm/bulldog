use lazy_static::lazy_static;
use x86_64::structures::gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector};
use x86_64::structures::tss::TaskStateSegment;
use x86_64::VirtAddr;
use crate::stack::{STACK, LAPIC_STACK};


pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;
pub const STACK_SIZE: usize = 128 * 1024;
pub const LAPIC_IST_INDEX: u16 = 1;


    lazy_static! {
    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();

        // Double fault stack
        let df_stack_start = VirtAddr::from_ptr(unsafe { core::ptr::addr_of!(STACK.0) });
        let df_stack_end = df_stack_start + STACK_SIZE;
        assert_eq!(df_stack_end.as_u64() % 16, 0);
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = df_stack_end;

        // LAPIC timer stack
        let lapic_stack_start = VirtAddr::from_ptr(unsafe { core::ptr::addr_of!(LAPIC_STACK.0) });
        let lapic_stack_end = lapic_stack_start + STACK_SIZE;
        assert_eq!(lapic_stack_end.as_u64() % 16, 0);
        tss.interrupt_stack_table[LAPIC_IST_INDEX as usize] = lapic_stack_end;

        tss
    };
}


lazy_static! {
    static ref GDT: (GlobalDescriptorTable, Selectors) = {
        let mut gdt = GlobalDescriptorTable::new();
        let code_selector = gdt.add_entry(Descriptor::kernel_code_segment());
        //let data_selector = gdt.add_entry(Descriptor::UserSegment(0));
        let data_selector = gdt.add_entry(Descriptor::kernel_data_segment());
        let tss_selector = gdt.add_entry(Descriptor::tss_segment(&TSS));
        
        (
            gdt,
            Selectors {
                code_selector,
                data_selector,
                tss_selector,
            },
        )
    };
}

struct Selectors {
    code_selector: SegmentSelector,
    data_selector: SegmentSelector,
    tss_selector: SegmentSelector,
}

pub fn init() {
    use x86_64::instructions::segmentation::{CS, DS, ES, SS, Segment};
    use x86_64::instructions::tables::load_tss;
    GDT.0.load();
   

    unsafe {
        CS::set_reg(GDT.1.code_selector);
        DS::set_reg(GDT.1.data_selector);
        ES::set_reg(GDT.1.data_selector);
        SS::set_reg(GDT.1.data_selector);
        load_tss(GDT.1.tss_selector);
    }
}