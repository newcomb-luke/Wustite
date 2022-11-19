MEMORY
{
  RAM : ORIGIN = 0xC0000000, LENGTH = 64K
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