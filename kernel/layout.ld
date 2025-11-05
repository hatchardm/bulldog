ENTRY(kernel_main)

SECTIONS {
    . = ALIGN(4K);

   .text : {
    KEEP(*(.text.entry_point))  /* ← preserves the kernel entrypoint */
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
