ENTRY(bulldog_entry)

SECTIONS {
    . = 1M;          /* or 0x100000 */

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
