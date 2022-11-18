MEMORY
{
  RAM : ORIGIN = 0x00100000, LENGTH = 64K
}

SECTIONS
{
  .text ORIGIN(RAM) :
  {
    *(.text .text.*);
  } > RAM
  .data : { *(.data) }
  .bss : { *(.bss) }
}