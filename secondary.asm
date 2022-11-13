[org 0x7c00 + 512 * 3]
[bits 64]

VIDEO_MEM equ 0xb8000
VID_ADDR_REG equ 0x03d4
VID_DATA_REG equ 0x03d5
CUR_LOC_LOW equ 0x0f
CUR_LOC_HIGH equ 0x0e

_start:
	cli

	mov edi, VIDEO_MEM
	mov rax, 0x0020002000200020
	mov ecx, 500
	rep stosq

	mov rax, 0x1f201f201f541f54
	mov [VIDEO_MEM], rax

	hlt

times 512 - ($ - $$) db 0
