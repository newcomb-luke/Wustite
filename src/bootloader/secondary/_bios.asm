[bits 16]

section _TEXT class=CODE

global __BIOS_Video_WriteCharTeletype
global __BIOS_Video_SetVideoMode

; args: character, page
__BIOS_Video_WriteCharTeletype:
	push bp
	mov bp, sp
	push bx

	; [bp + 4] - character
	; [bp + 6] - page

	mov ah, 0x0e
	mov al, [bp + 4]
	mov bh, [bp + 6]
	int 0x10

	pop bx
	mov sp, bp
	pop bp
	ret


; args: mode
__BIOS_Video_SetVideoMode:
	push bp
	mov bp, sp

	; [bp + 4] - mode
	
	; set video mode
	mov ah, 0x00
	mov al, [bp + 4]
	int 0x10

	mov sp, bp
	pop bp
	ret
