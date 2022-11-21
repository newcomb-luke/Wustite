ENTRY(_start)

origin = 0x00020000;

SECTIONS
{
  . = origin;
  .text : { *(.text) *(.text.*) }
  .data : { *(.data) *(.data.*) }
  .rodata : { *(.rodata) *(.rodata.*) }
  .bss : { *(.bss) *(.bss.*) }
}