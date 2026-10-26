use x86_64::structures::gdt::SegmentSelector;
use crate::gdt;


pub unsafe fn enter_user_mode(entry: u64, user_stack_top: u64) -> ! {
    let cs = gdt::user_code_selector().0 as u64;
    let ss = gdt::user_data_selector().0 as u64;

    // System V ABI: reserve red zone
    let rsp = user_stack_top - 128;

    // Enable interrupts (IF=1), reserved bit set
    let rflags = 0x202;

    core::arch::asm!(
        // Load usermode data segments
        "mov ds, {ss}",
        "mov es, {ss}",
        "mov fs, {ss}",
        "mov gs, {ss}",

        // Prepare iretq frame
        "push {ss}",
        "push {rsp}",
        "push {rflags}",
        "push {cs}",
        "push {rip}",

        // Switch GS base (kernel → user)
        "swapgs",

        // Enter usermode
        "iretq",

        ss     = in(reg) ss,
        rsp    = in(reg) rsp,
        rflags = in(reg) rflags,
        cs     = in(reg) cs,
        rip    = in(reg) entry,
        options(noreturn),
    );
}
