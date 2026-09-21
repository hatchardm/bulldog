use x86_64::structures::gdt::SegmentSelector;
use crate::gdt;


pub unsafe fn enter_user_mode(entry: u64, user_stack_top: u64) -> ! {
    let user_cs: SegmentSelector = gdt::user_code_selector();
    let user_ds: SegmentSelector = gdt::user_data_selector();

    let cs = user_cs.0 as u64;
    let ss = user_ds.0 as u64;
    let rip = entry;
    let rsp = user_stack_top;
    let rflags = 0x202; // IF=1

    core::arch::asm!(
        "push {ss}",
        "push {rsp}",
        "push {rflags}",
        "push {cs}",
        "push {rip}",
        "iretq",
        ss     = in(reg) ss,
        rsp    = in(reg) rsp,
        rflags = in(reg) rflags,
        cs     = in(reg) cs,
        rip    = in(reg) rip,
        options(noreturn),
    );
}