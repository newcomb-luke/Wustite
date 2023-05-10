.section .entry
.global entry

entry:
	# the boot drive number is in dl
    # mov [g_BootDrive], dl

    xor ax, ax
    mov ds, ax
    mov es, ax
    mov ss, ax
    mov sp, 0xFFF0
	mov bp, sp

    # Disable interrupts
    cli

    # Enable the A20 line
    # call __enable_a20
    # cmp ax, 0
    # jne halt

    call main

halt:
    cli
    hlt
    jmp halt
