; dx: hex value, 16 bits
print_hex:
	pusha
	push bp
	xor bp, bp

	mov si, HEX_PATTERN

	mov cl, 12
	mov bp, 2

	.loop:
		cmp bp, 6
		je .print

		mov bx, dx
		shr bx, cl
		and bx, 0x000F
		mov bx, [bx + HEX_TABLE]

		mov [bp + HEX_PATTERN], bl

		sub cl, 4
		add bp, 1
		jmp .loop
	.print:

	call puts

	pop bp
	popa
	ret

HEX_PATTERN: db "0x****", 0x0a, 0x0d, 0
HEX_TABLE: db "0123456789abcdef"

%include "./stage1-util.s"
