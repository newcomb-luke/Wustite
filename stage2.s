[org 0x7c00 + 512]
[bits 16]

section .text

global start

start:
	mov si, ALIVE_MSG
	call puts

	mov dx, [0x7c00 + 510]
	call print_hex

	jmp $

%include "./util.s"

ALIVE_MSG: db "Reached stage 2", 0x0a, 0x0d, 0

times 512 * 4 db 0
