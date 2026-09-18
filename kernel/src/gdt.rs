#![allow(dead_code)]

use alloc::boxed::Box;
use spin::Mutex;
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

/// Convenience container for GDT segment selectors.
struct Selectors {
    code_selector: SegmentSelector,
    data_selector: SegmentSelector,
    tss_selector: SegmentSelector,
    user_code_selector: SegmentSelector,
    user_data_selector: SegmentSelector,
}

/// Global storage for selectors only (no `static mut` GDT/TSS).
static SELECTORS: Mutex<Option<Selectors>> = Mutex::new(None);

/// Load the GDT and activate segment registers and TSS.
/// Safety: must be called once during early kernel init, after paging + heap.
pub fn init() {
    use x86_64::instructions::segmentation::{CS, DS, ES, SS, Segment};
    use x86_64::instructions::tables::load_tss;

    //
    // 1. Build TSS with IST stacks
    //
    let mut tss = TaskStateSegment::new();

    // Double fault stack: use end address as top of stack.
    let df_stack_start = VirtAddr::from_ptr(unsafe { core::ptr::addr_of!(STACK.0) });
    let df_stack_end = df_stack_start + STACK_SIZE;
    tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = df_stack_end;

    // LAPIC/IST stack: used for page fault and LAPIC timer handlers.
    let lapic_stack_start = VirtAddr::from_ptr(unsafe { core::ptr::addr_of!(LAPIC_STACK.0) });
    let lapic_stack_end = lapic_stack_start + STACK_SIZE;
    tss.interrupt_stack_table[LAPIC_IST_INDEX as usize] = lapic_stack_end;

    // Leak TSS to get a true 'static reference (safe: one-time kernel object).
    let tss_static: &'static TaskStateSegment = Box::leak(Box::new(tss));

    //
    // 2. Build GDT with kernel + user segments + TSS
    //
    let mut gdt = GlobalDescriptorTable::new();

    let code_selector = gdt.add_entry(Descriptor::kernel_code_segment());
    let data_selector = gdt.add_entry(Descriptor::kernel_data_segment());
    let tss_selector  = gdt.add_entry(Descriptor::tss_segment(tss_static));

    // User-mode segments (for future syscall/userspace work)
    let user_code_selector = gdt.add_entry(Descriptor::user_code_segment());
    let user_data_selector = gdt.add_entry(Descriptor::user_data_segment());

    let selectors = Selectors {
        code_selector,
        data_selector,
        tss_selector,
        user_code_selector,
        user_data_selector,
    };

    //
    // 3. Leak GDT to get a 'static reference and load it
    //
    let gdt_static: &'static GlobalDescriptorTable = Box::leak(Box::new(gdt));
    gdt_static.load();

    unsafe {
        CS::set_reg(selectors.code_selector);
        DS::set_reg(selectors.data_selector);
        ES::set_reg(selectors.data_selector);
        SS::set_reg(selectors.data_selector);
        load_tss(selectors.tss_selector);
    }

    //
    // 4. Store selectors for later access
    //
    let mut guard = SELECTORS.lock();
    *guard = Some(selectors);
}

pub fn user_code_selector() -> SegmentSelector {
    let guard = SELECTORS.lock();
    guard.as_ref().expect("GDT selectors not initialized").user_code_selector
}

pub fn user_data_selector() -> SegmentSelector {
    let guard = SELECTORS.lock();
    guard.as_ref().expect("GDT selectors not initialized").user_data_selector
}

pub fn kernel_code_selector() -> SegmentSelector {
    let guard = SELECTORS.lock();
    guard.as_ref().expect("GDT selectors not initialized").code_selector
}

pub fn kernel_data_selector() -> SegmentSelector {
    let guard = SELECTORS.lock();
    guard.as_ref().expect("GDT selectors not initialized").data_selector
}

