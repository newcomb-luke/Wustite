ENTRY(_start)

origin = 0x00020000;

SECTIONS
{
  . = origin;
  __KERNEL_LOAD_LOC = .;
  .text : { *(.text) *(.text.*) }
  .data : { *(.data) *(.data.*) }
  .rodata : { *(.rodata) *(.rodata.*) }
  .bss : { *(.bss) *(.bss.*) }
  __KERNEL_LOAD_END = .;
}