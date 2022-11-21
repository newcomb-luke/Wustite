[org 0x7c00]
[bits 16]

; BEGIN FAT12 HEADER

; BIOS Parameter Block

; Jump over FAT BPB data
jmp short _start
nop

bdb_oem:			db "MSWIN4.1"
bdb_bytes_per_sector: 		dw 512
bdb_sectors_per_cluster: 	db 1
bdb_reserved_sectors: 		dw 1
bdb_fat_count:			db 2
bdb_dir_entries_count:		dw 0xe0
bdb_total_sectors:		dw 2880 			; 1.44 MB
bdb_media_descriptor_type: 	db 0xf0 			; 3.5 inch floppy disk
bdb_sectors_per_fat: 		dw 9
bdb_sectors_per_track: 		dw 18
bdb_head_count:			dw 2
bdb_hidden_sectors: 		dd 0
bdb_large_sectors:		dd 0

; Extended Boot Record
ebr_drive_number: 		db 0x00 			; 0x00 floppy, 0x80 hdd
				db 0    			; Reserved
ebr_signature: 			db 0x29
ebr_volume_id:			db 0x12, 0x34, 0x56, 0x78
ebr_volume_label:		db "WUSTITEBOOT" 		; Must be 11 bytes long, padded with spaces
ebr_system_id:			db "FAT12   " 			; Must be 8 bytes long, padded with spaces

; END FAT12 HEADER

_start:
	; Disable interrupts
	cli

	; Zero the segment registers
	jmp 0x0000:.zero_seg
.zero_seg:
	xor ax, ax
	mov ss, ax
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    ; Put the stack in the top section above where we are loaded
    mov sp, 0x7bff

	; BIOS should set dl to the drive number we have booted from
	; store it
	mov [ebr_drive_number], dl

    cld

	; Read the other drive parameters from the BIOS
	push es

	mov ah, 0x08
	int 0x13
	jc disk_read_error

	pop es

	; bottom 6 bits of cl store the sector count
	and cl, 0x3f
	xor ch, ch
	mov [bdb_sectors_per_track], cx

	; dh: head count - 1
	inc dh
	mov [bdb_head_count], dh

	pusha

	; Clear the screen
	mov ah, 0x00
	; text mode 80x25 16 colors
	mov al, 0x03
	int 0x10

	popa
	
	; Print our startup message
	mov si, STARTUP_MSG
	call b_puts
	
	; Reset the disk controller, just in case
	mov dl, [ebr_drive_number]
	call b_reset_disk

	; calculate LBA of root directory:
	;	= reserved = FATs * sectors per FAT
	mov ax, [bdb_sectors_per_fat]
	mov bl, [bdb_fat_count]
	xor bh, bh
	mul bx
	add ax, [bdb_reserved_sectors]
	push ax ; LBA is stored on the stack

	; calculate the size of the root directory in sectors:
	;	= (32 * number of entries) / bytes per sector
	mov ax, [bdb_dir_entries_count]
	shl ax, 5			; ax = ax * 32
	xor dx, dx
	div word [bdb_bytes_per_sector]
	
	test dx, dx
	jz .root_dir_after
	; We need to do this because if the remainder != 0,
	; then we round up to the nearest sector
	inc ax

.root_dir_after:
	; read root directory
	; cl = number of sectors to read, the size of the root directory
	mov cl, al
	pop ax	; LBA of root directory
	mov dl, [ebr_drive_number]
	mov bx, buffer
	call b_read_disk_lba

	; search for secboot.bin
	xor bx, bx
	mov di, buffer

.search_for_secondary:
	mov si, SECONDARY_FILE_NAME
	mov cx, 11

	push di
	repe cmpsb
	pop di
	je .found_secondary

	add di, 32
	inc bx
	cmp bx, [bdb_dir_entries_count]
	jl .search_for_secondary

	mov si, SECONDARY_NOT_FOUND_MSG
	call b_puts
	jmp halt

.found_secondary:
	
	; di should still have the address of the entry
	mov ax, [di + 26] ; lower cluster number field offset 26
	mov [SECONDARY_CLUSTER_PTR], ax

	; load FAT
	mov ax, [bdb_reserved_sectors]
	mov bx, buffer
	mov cl, [bdb_sectors_per_fat]
	mov dl, [ebr_drive_number]
	call b_read_disk_lba

	; read through all of the cluster entries
	mov bx, SECONDARY_LOAD_SEGMENT
	mov es, bx
	mov bx, SECONDARY_LOAD_OFFSET

.load_secondary_loop:
	; read the next cluster
	mov ax, [SECONDARY_CLUSTER_PTR]
	; TODO FIXME
	; This is hard-coded to our FAT12 disk, very bad, but unfortunately
	; it works :(
	add ax, 31

	mov cl, 1
	mov dl, [ebr_drive_number]
	call b_read_disk_lba

	add bx, [bdb_bytes_per_sector]

	; TODO FIXME
	; this will wrap back around and start overwriting
	; if our secondary bootloader is larger than 64k
	mov ax, [SECONDARY_CLUSTER_PTR]
	mov cx, 3
	mul cx
	mov cx, 2
	div cx
	; ax = index of entry in FAT, dx = cluster % 2

	mov si, buffer
	add si, ax
	mov ax, [ds:si]

	or dx, dx
	jz .even

.odd:
	shr ax, 4
	jmp .next_cluster_after

.even:
	and ax, 0x0FFF

.next_cluster_after:
	cmp ax, 0x0FF8 ; 0x0FF8 marks the end of the cluster chain
	jae .read_finish

	mov [SECONDARY_CLUSTER_PTR], ax
	jmp .load_secondary_loop

.read_finish:
	; Provide the boot drive number in dl the register
	mov dl, [ebr_drive_number]
	; Set the segment registers
	mov ax, SECONDARY_LOAD_SEGMENT
	mov ds, ax
	mov es, ax

	jmp SECONDARY_LOAD_SEGMENT:SECONDARY_LOAD_OFFSET
	
	; just in case
	jmp halt

; ax: LBA address
; returns:
; 	cx (bits 0-5): sector number
;	cx (bits 6-15): cylinder number
;	dh: head
lba_to_chs:
	push ax
	push dx

	xor dx, dx				; dx = 0
	div word [bdb_sectors_per_track]	; ax = LBA / sectors per track
						; dx = LBA % sectors per track 
	inc dx 					; dx = (LBA % sectors per track) + 1 = sector (starts at 1)
	mov cx, dx				; cx = sector

	xor dx, dx				; dx = 0
	div word [bdb_head_count]		; ax = (LBA / sectors per track) / number of heads
						; dx = (LBA % sectors per track) / number of heads = head
	mov dh, dl				; dh = dx = head
	mov ch, al 				; ch = cylinder (lower 8 bits)
	shl ah, 6				; max size for sector number is 6
	or cl, ah				; set cl upper 2 bits of cl with upper 2 bits of cylinder

	pop ax		; This was the dx register
	mov dl, al	; but we return using dh
	pop ax
	ret

; ax: LBA address
; cl: number of sectors to read (up to 128)
; dl: drive number
; es:bx: memory address to store the read data
b_read_disk_lba:
	push ax
	push bx
	push cx
	push dx
	push di

	push cx
	call lba_to_chs		; returns in cx and dh
	pop ax			; al = number of sectors to read

	mov ah, 0x02		; int13 "read from disk"

	; The BIOS docs say to retry reading at least 3 times
	mov di, 3

.retry:
	pusha
	stc			; Set the carry flag
	int 0x13		; If carry flag = 0, success
	jnc .done

	; fail
	popa
	call b_reset_disk	; dl is already the drive number

	dec di
	test di, di
	jnz .retry

.fail:
	jmp disk_read_error

.done:
	popa

	pop di
	pop dx
	pop cx
	pop bx
	pop ax

	ret

; dl: drive number
b_reset_disk:
	pusha

	mov ah, 0x00 ; "reset"
	stc
	int 0x13

	jc disk_read_error

	popa
	ret

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

disk_read_error:
	mov si, DISK_READ_ERROR_MSG
	call b_puts
	jmp halt

halt:
	cli
	hlt
	jmp halt

STARTUP_MSG: 			db "Boot!", 0x0a, 0x0d, 0
DISK_READ_ERROR_MSG: 		db "Disk read error", 0x0a, 0x0d, 0
SECONDARY_FILE_NAME: 		db "SECBOOT BIN"
SECONDARY_NOT_FOUND_MSG: 	db "SECBOOT.BIN not found", 0x0a, 0x0d, 0

SECONDARY_CLUSTER_PTR:	dw 0

SECONDARY_LOAD_SEGMENT 	equ 0x0000
SECONDARY_LOAD_OFFSET 	equ 0x0500

; Padding
times 510 - ($ - $$) db 0
; Magic number to declare a boot sector
dw 0xaa55

buffer: