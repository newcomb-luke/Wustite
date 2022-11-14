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
        cld

	; BIOS should set dl to the drive number we have booted from
	; store it
	mov [ebr_drive_number], dl
	
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

	; Read 1 sector from the disk
	mov ax, 1 ; LBA = 1, second sector on the disk
	mov cl, 1 ; read one sector
	mov bx, 0x7c00 + 512 ; location right after the boot sector
	call b_read_disk_lba

	mov si, DISK_READ_SUCCESS_MSG
	call b_puts
	
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

disk_read_error:
	mov si, DISK_READ_ERROR_MSG
	call b_puts
	jmp halt

halt:
	cli
	hlt
	jmp halt

%include "./bootloader/src/bios-utils.asm"

STARTUP_MSG: 		db "Bootloader start", 0x0a, 0x0d, 0
DISK_READ_ERROR_MSG: 	db "Disk read error", 0x0a, 0x0d, 0
DISK_READ_SUCCESS_MSG:	db "Successfully read disk", 0x0a, 0x0d, 0

; Padding
times 510 - ($ - $$) db 0
; Magic number to declare a boot sector
dw 0xaa55

; _stage2:
;     mov si, SECTOR_2_MSG
;     call b_puts
; 
;     call test_a20
;     cmp ax, 0
;     je a20_already
; 
;     mov si, A20_ENABLING_TRY
;     call b_puts
; 
;     call try_enable_a20
;     cmp ax, 0
;     je a20_success
;     jmp a20_failure
; 
;     .after_a20:
; 
;     jmp $
; 
;     mov si, LONG_MODE_TRY
;     call b_puts
; 
;     call check_long_mode
;     cmp ax, 0
; 
;     je .enable_long_mode
;     jmp long_mode_failure
; 
;     .enable_long_mode:
;     mov eax, longmode
;     call enable_long_mode
; 
;     jmp halt
; 
; a20_already:
;     mov si, A20_ENABLED_ALREADY
;     call b_puts
;     jmp _stage2.after_a20
; 
; a20_success:
;     mov si, A20_SUCCESS
;     call b_puts
;     jmp _stage2.after_a20
; 
; a20_failure:
;     mov si, A20_FAILURE
;     call b_puts
;     jmp halt
; 
; long_mode_failure:
;     mov si, LONG_MODE_UNAVAILABLE
;     call b_puts
;     jmp halt
; 
; halt:
;     jmp $
; 
; %include "./bootloader/src/stage2-bios-utils.asm"
; %include "./bootloader/src/a20.asm"
; %include "./bootloader/src/gdt.asm"
; %include "./bootloader/src/long-mode.asm"
; 
; SECTOR_2_MSG: db "Reached sector 2", 0x0a, 0x0d, 0
; A20_ENABLED_ALREADY: db "A20 line was enabled at boot", 0x0a, 0x0d, 0
; A20_ENABLING_TRY: db "Attempting to enable A20 line", 0x0a, 0x0d, 0
; A20_SUCCESS: db "A20 line successfully enabled", 0x0a, 0x0d, 0
; A20_FAILURE: db "ERROR: Failed to enable A20 line", 0x0a, 0x0d, 0
; LONG_MODE_TRY: db "Attempting to enable long mode", 0x0a, 0x0d, 0
; LONG_MODE_SUCCESS: db "Successfully enabled long mode", 0x0a, 0x0d, 0
; LONG_MODE_UNAVAILABLE: db "Long mode is not supported on this machine. Cannot boot.", 0x0a, 0x0d, 0
; 
; [bits 64]
; 
; longmode:
; 	cli
; 
; 	jmp 0x7c00 + 512 * 3
; 
; 	hlt
; 
; times (512 * 2) - (($ - $$) - 512) db 0
