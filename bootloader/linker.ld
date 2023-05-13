ENTRY(entry)

origin = 0x00000500;

SECTIONS {
    . = origin;

    .text : { *(.entry) *(.text .text.*) }

    .bss : {
        __bss_start = .;
        *(.bss .bss.*)
        __bss_end = .;
    }

    .rodata : { *(.rodata .rodata.*) }
    .data : { *(.data .data.*) }
    .eh_frame : { *(.eh_frame .eh_frame.*) }
    .eh_frame_hdr : { *(.eh_frame_hdr .eh_frame_hdr.*) }
}
