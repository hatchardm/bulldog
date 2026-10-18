ENTRY(_start)

SECTIONS
{
  . = 0x100000000;   /* 4 GiB — above MAP_SIZE */

  .text : {
    *(.text .text.*)
  }
  .rodata : {
    *(.rodata .rodata.*)
  }
  .data : {
    *(.data .data.*)
  }
  .bss : {
    *(.bss .bss.*)
  }
}