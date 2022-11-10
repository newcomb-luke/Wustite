[org 0x7c00]

mov si, STARTUP_MSG
call puts

mov al, 0x01 ; We're reading only one sector
mov cl, 0x02 ; We are in the first sector, the next one is sector 2
mov bx, 0x7c00 + 512 ; The location right after this boot sector
call read_disk

jmp first

jmp $

; null-terminated string pointer in si
puts:
	pusha
	.loop:
		mov al, [si]
		cmp al, 0
		je .end
		call putc
		add si, 1
		jmp .loop
	.end:
	popa
	ret

; char stored in al
putc:
	pusha
	mov ah, 0x0e
	int 0x10
	popa
	ret

; al: number of sectors to read. Begins at 1
; cl: sector to begin reading at
; bx: location to load the read data into
;
; Halts and prints error message on failure
read_disk:
	pusha

	mov ah, 0x02 ; We want to read the disk
	mov dl, 0x80 ; We are going to be read as a "hard drive" from Qemu
	mov ch, 0x00 ; First cylinder
	mov dh, 0x00 ; First head
	; mov al, 0x01 ; We're reading only one sector
	; mov cl, 0x02 ; We are in the first sector, the next one is sector 2

	; Zero the es register
	push bx
	mov bx, 0
	mov es, bx
	pop bx

	; mov bx, 0x7c00 + 512 ; The location right after this boot sector

	int 0x13

	jc .error

	popa
	ret

	.error:
		mov si, DISK_READ_ERROR_MSG
		call puts
		jmp $

STARTUP_MSG: db "Starting bootloader", 0x0a, 0x0d, 0
DISK_READ_ERROR_MSG: db "ERROR: Could not load kernel from disk", 0x0a, 0x0d, 0

; Padding
times 510-($-$$) db 0
; Magic number to declare a boot sector
dw 0xaa55

first:
mov si, ALIVE_MSG
call puts

jmp $

ALIVE_MSG: db "I'm alive!", 0x0a, 0x0d, 0

times 512 db 0
