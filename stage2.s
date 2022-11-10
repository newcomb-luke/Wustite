[org 0x7c00 + 512]
[bits 16]

section .text

global start

start:
	mov si, ALIVE_MSG
	call puts

	jmp $

%include "./util.s"

ALIVE_MSG: db "Reached stage 2", 0x0a, 0x0d, 0

times 512 db 0
