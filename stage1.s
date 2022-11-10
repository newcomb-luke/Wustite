[org 0x7c00]
[bits 16]

section .text

global init

init:
	; All kinds of BIOS correction stuff

	cli ; Stops interrupts

	; Zeros the segment registers
	jmp 0x0000:zero_seg

	zero_seg:
		xor ax, ax
		mov ss, ax
		mov ds, ax
		mov es, ax
		mov fs, ax
		mov gs, ax
		mov sp, init
		cld

	sti ; Resume interrupts

	; Reset the disk, just in case
	push ax
	xor ax, ax   ; 0 means reset disks
	mov dl, 0x80 ; Hard drive
	int 0x13
	pop ax

	jmp start

start:
	mov si, STARTUP_MSG
	call puts

	mov al, 0x01 ; We're reading only one sector
	mov cl, 0x02 ; We are in the first sector, the next one is sector 2
	mov bx, 0x7c00 + 512 ; The location right after this boot sector
	mov si, DISK_READ_ERROR_MSG
	call read_disk

	jmp bx

%include "./util.s"

STARTUP_MSG: db "Reached stage 1", 0x0a, 0x0d, 0
DISK_READ_ERROR_MSG: db "ERROR: Could not load sector from disk", 0x0a, 0x0d, 0

; Padding
times 510-($-$$) db 0
; Magic number to declare a boot sector
dw 0xaa55
