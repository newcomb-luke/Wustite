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
