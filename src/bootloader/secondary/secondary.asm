[bits 16]

[section _ENTRY class=CODE]

extern _cstart_
global _start

; The secondary-stage bootloader, still in 16 bit real mode
; with BIOS. Still using the same stack as the first stage
; bootloader (although it should be empty).

_start:
	cli
	mov ax, ds
	mov ss, ax
	mov sp, 0
	mov bp, sp
	sti

	; the boot drive number is in dl
	xor dh, dh
	push dx
	call _cstart_

	jmp halt

halt:
	cli
	hlt
	jmp halt

; null-terminated string pointer in si
b_puts:
	pusha
	.loop:
		mov al, [si]
		cmp al, 0
		je .end
		call b_putc
		add si, 1
		jmp .loop
	.end:
	popa
	ret

; char stored in al
b_putc:
	pusha
	mov ah, 0x0e
	int 0x10
	popa
	ret
