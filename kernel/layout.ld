ENTRY(kernel_entry)

SECTIONS {

    .text : {
        KEEP(*(.text.entry_point))
        *(.text*)
    }

    .rodata : { *(.rodata*) }
    .data   : { *(.data*) }
    .bss    : { *(.bss*) *(COMMON) }

    .stack ALIGN(4K) : {
        KEEP(*(.stack))
    }
}

