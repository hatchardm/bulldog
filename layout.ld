ENTRY(kernel_main)

SECTIONS {
    . = 1M;

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

