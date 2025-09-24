SECTIONS {
    . = ALIGN(4K);

    .text : {
        *(.text*)
    }

    .rodata : {
        *(.rodata*)
    }

    .data : {
        *(.data*)
    }

    .bss : {
        *(.bss*)
        *(COMMON)
    }

    .stack ALIGN(4K) : {
        KEEP(*(.stack))
    }
}
