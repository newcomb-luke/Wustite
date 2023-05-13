[bits 16]

section .entry

global entry
extern bootloader_entry
extern __bss_start
extern __bss_end

entry:
    xor ax, ax
    mov ds, ax
    mov es, ax
    mov ss, ax
    mov sp, 0xFFF0
	mov bp, sp

    ; Zero the .bss section
    mov edi, __bss_start
    mov ecx, __bss_end
    sub ecx, edi
    mov al, 0
    cld
    rep stosb

	call bootloader_entry

	jmp $
