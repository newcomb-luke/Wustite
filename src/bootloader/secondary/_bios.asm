[bits 16]

section _TEXT class=CODE

global __BIOS_Video_WriteCharTeletype
global __BIOS_Video_SetVideoMode
global __BIOS_Drive_Reset
global __BIOS_Drive_GetParams
global __BIOS_Drive_ReadSectors
global __U4D
global __U4M

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

; args: drive number
; returns 0 if successful, nonzero otherwise
__BIOS_Drive_Reset:
    push bp
    mov bp, sp
    push dx

    ; [bp + 4] - drive number
    mov ah, 0x00 ; "reset"
    mov dl, [bp + 4]
    stc
    int 0x13
    jc .fail
    xor ax, ax
    jmp .done

.fail:
    mov ax, 1

.done:
    pop dx
    mov sp, bp
    pop bp
    ret

;
; uint16_t _cdecl _BIOS_Drive_GetParams(uint8_t driveNumber,
;                                       uint8_t* driveType,
;                                       uint8_t* maxHeadOut,
;                                       uint16_t* maxCylinderOut,
;                                       uint8_t* maxSectorOut);
;
__BIOS_Drive_GetParams:
    push bp
    mov bp, sp
    push bx
    push si

    ; [bp + 4] - drive number
    ; [bp + 6] - drive type ptr
    ; [bp + 8] - max head ptr
    ; [bp + 10] - max cylinder ptr
    ; [bp + 12] - max sector ptr

    push es
    push di
    ; ES:DI = 0x0000:0x0000
    xor di, di
    mov es, di
    mov ah, 0x08
    mov dl, [bp + 4]
    stc
    int 0x13
    pop di
    pop es

    jc .fail

    ; BL = drive type (AT/PS2 floppies only) (see #00242)
    ; CH = low eight bits of maximum cylinder number
    ; CL = maximum sector number (bits 5-0)
    ; high two bits of maximum cylinder number (bits 7-6)
    ; DH = maximum head number
    ; DL = number of drives

    ; Store the drive type
    mov si, [bp + 6]
    mov [si], bl

    ; Store the max head number
    mov si, [bp + 8]
    mov [si], dh

    ; Store the max cylinder number
    mov bl, ch
    mov bh, cl
    shr bh, 6
    mov si, [bp + 10]
    mov [si], bx

    ; Store the max sector number
    and cl, 0b00111111 ; sector number is in bits 5-0
    mov si, [bp + 12]
    mov [si], cl

    xor ax, ax ; success
    jmp .done

.fail:
    mov ax, 1

.done:
    pop si
    pop bx
    mov sp, bp
    pop bp
    ret

;
; uint16_t _cdecl _BIOS_Drive_ReadSectors(uint8_t driveNumber,
;                                         uint8_t head,
;                                         uint16_t cylinder,
;                                         uint8_t sector,
;                                         uint8_t sectorCount,
;                                         uint8_t far* dataDestination);
;
__BIOS_Drive_ReadSectors:
    push bp
    mov bp, sp

    push bx
    push es

    ; [bp + 4] - drive number
    ; [bp + 6] - head
    ; [bp + 8] - cylinder
    ; [bp + 10] - sector
    ; [bp + 12] - sector read count
    ; [bp + 14] - data destination (low 16 bits)
    ; [bp + 16] - data destination (high 16 bits)

    ; AL = number of sectors to read (must be nonzero)
    ; CH = low eight bits of cylinder number
    ; CL = sector number 1-63 (bits 0-5)
    ; high two bits of cylinder (bits 6-7, hard disk only)
    ; DH = head number
    ; DL = drive number (bit 7 set for hard disk)
    ; ES:BX -> data buffer

    ; Set drive number
    mov dl, [bp + 4]

    ; Set cylinder number
    mov ch, [bp + 8]
    mov cl, [bp + 9]
    shl cl, 6

    ; Set head number
    mov dh, [bp + 6]

    ; Set sector number
    mov al, [bp + 10]
    and al, 0b00111111 ; Clear top bits of sector number
    or cl, al

    ; Set number of sectors to read
    mov al, [bp + 12]

    ; Set destination data buffer
    mov bx, [bp + 16]

    ; mov bl, bh
    ; call ___print_hex
    ; mov bx, [bp + 16]
    ; and bx, 0x0F
    ; call ___print_hex
    ; mov bx, [bp + 16]

    mov es, bx
    mov bx, [bp + 14]

    ; mov bl, bh
    ; call ___print_hex
    ; mov bx, [bp + 14]
    ; and bx, 0x0F
    ; call ___print_hex
    ; mov bx, [bp + 14]

	; jmp $

    mov ah, 0x02
    stc
    int 0x13
    jnc .success

.fail:
    pop es
    pop bx
    mov ax, 1
    jmp .done

.success:
    pop es
    pop bx
    xor ax, ax ; success

.done:

    mov sp, bp
    pop bp
    ret

; bl - byte
___print_hex:
    push ax
    push bx
    push cx

    mov cl, bl
    shr bl, 4
    and bx, 0x0f

    pusha
	mov ah, 0x0e
	mov al, [HEX_MAP + bx]
	mov bh, 0
	int 0x10
	popa

	mov bl, cl
	and bx, 0x0F

    pusha
	mov ah, 0x0e
	mov al, [HEX_MAP + bx]
	mov bh, 0
	int 0x10
	popa

    pop cx
    pop bx
    pop ax
    ret

HEX_MAP: db "0123456789abcdef"

;
; void _cdecl _x86_div64_32(uint64_t dividend,
;                           uint32_t divisor,
;                           uint64_t* quotientOut,
;                           uint32_t* remainderOut);
;
__x86_div64_32:
    push bp
    mov bp, sp
    push bx

    ; divide upper 32 bits
    mov eax, [bp + 8]
    mov ecx, [bp + 12]
    xor edx, edx
    div ecx
    ; eax: quotient, edx: remainder
    mov bx, [bp + 16]
    mov [bx + 4], eax

    mov eax, [bp + 4]
    div ecx

    mov [bx], eax
    mov bx, [bp + 18]
    mov [bx], edx

    pop bx
    mov sp, bp
    pop bp
    ret

; unsigned 4-byte divide
; input:
; dx;ax dividend
; cx;bx divisor
; output:
; dx;ax quotient
; cx;bx remainder
__U4D:
    shl edx, 16
    mov dx, ax
    mov eax, edx
    xor edx, edx

    shl ecx, 16
    mov cx, bx

    div ecx
    mov ebx, edx
    mov ecx, edx
    shr ecx, 16

    mov edx, eax
    shr edx, 16

    ret

; 4 byte multiply
; input:
; dx;ax - integer 1
; cx;bx - integer 2
; output:
; dx;ax - product
__U4M:
    shl edx, 16
    mov dx, ax
    mov eax, edx

    shl ecx, 16
    mov cx, bx

    mul ecx

    mov edx, eax
    shr edx, 16

    ret