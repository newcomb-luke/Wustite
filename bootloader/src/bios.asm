[bits 32]

section .text

global _BIOS_Video_WriteCharTeletype
global _BIOS_Video_SetVideoMode
global _BIOS_Drive_Reset
global _BIOS_Drive_GetParams
global _BIOS_Drive_ReadSectors

%macro x86_EnterRealMode 0
    [bits 32]
    cli
    jmp word 0x18:.pmode16

.pmode16:
    [bits 16]
    ; Disable protected mode bit from cr0
    mov eax, cr0
    and al, ~1
    mov cr0, eax

    ; Jump to real mode again
    jmp word 0x00:.realmode
.realmode:
    [bits 16]
    ; Set up real mode segments
    xor ax, ax
    mov ds, ax
    mov ss, ax

    ; Re-enable interrupts
    sti
%endmacro

%macro x86_EnterProtectedMode 0
    [bits 16]
    cli
    ; Set protected mode bit in control register 0
    mov eax, cr0
    or al, 1
    mov cr0, eax

    jmp dword 0x08:.pmode32

.pmode32:
    [bits 32]
    ; Initialize segment registers
    mov ax, 0x10 ; Offset 16 into the GDT
    mov ds, ax
    mov ss, ax
%endmacro

; Convert linear address to real mode segment address
; Args:
;   1 - (in) linear address
;   2 - (out) target segment register
;   3 - target 32 bit register to use (eg. eax)
;   4 - target lower 16-bit half of argument 3 (eg. ax)
%macro LinearToSegmentOffset 4
    mov %3, %1 ; Move linear address into intermediary 32 bit register
    shr %3, 4 ; Shift eax to the right 4 bits, this gets it into the segment-offset form
    mov %2, %4
    mov %3, %1
    and %3, 0xF
%endmacro

; int32_t _BIOS_Memory_GetNextSegment(SMAPEntry* entry, uint32_t* contID);
global _BIOS_Memory_GetNextSegment
_BIOS_Memory_GetNextSegment:
    [bits 32]
    push ebp
    mov ebp, esp

    ; [ebp + 8] - entry pointer
    ; [ebp + 12] - continuation id pointer

	x86_EnterRealMode
	[bits 16]

	push ebx
	push ecx
	push edx
	push esi
	push edi
	push ds
	push es

    LinearToSegmentOffset [bp + 12], ds, esi, si
    mov ebx, [ds:si]
    ; Magic number that says, yes we want this data
    mov edx, 0x534D4150
    LinearToSegmentOffset [bp + 8], es, edi, di
    mov [es:di + 20], dword 1	; force a valid ACPI 3.X entry
    mov ecx, 24
    mov eax, 0x0000E820
    int 0x15
    jc .failed
    mov edx, 0x534D4150
    cmp eax, edx		; on success, eax must have been reset to "SMAP"
    jne .failed
    jmp .success

.failed:
    mov eax, -1
    jmp .done
.success:
    ; ecx contains the number of bytes read into our structure
    mov eax, ecx
    ; Set our continuation id value
    mov [ds:si], ebx
.done:
    pop es
    pop ds
    pop edi
    pop esi
    pop edx
    pop ecx
    pop ebx

    push eax

	x86_EnterProtectedMode
	[bits 32]

	pop eax

    mov esp, ebp
    pop ebp
    ret

; args: character
_BIOS_Video_WriteCharTeletype:
    [bits 32]
	push ebp
	mov ebp, esp
	push eax
    push ebx

    mov bl, [ebp + 8]

	x86_EnterRealMode
	[bits 16]

	; [bp + 8] - character

	mov ah, 0x0e
	mov al, bl
	xor bx, bx
	int 0x10

	x86_EnterProtectedMode
	[bits 32]

    pop ebx
    pop eax
	mov esp, ebp
	pop ebp
	ret


; args: mode
_BIOS_Video_SetVideoMode:
    [bits 32]
	push ebp
	mov ebp, esp

	x86_EnterRealMode
	[bits 16]

	; [bp + 8] - mode
	
	; set video mode
	mov ah, 0x00
	mov al, [bp + 8]
	int 0x10

	x86_EnterProtectedMode
	[bits 32]

	mov esp, ebp
	pop ebp
	ret

; args: drive number
; returns 0 if successful, nonzero otherwise
_BIOS_Drive_Reset:
    [bits 32]
    push ebp
    mov ebp, esp
    push edx

    x86_EnterRealMode
    [bits 16]

    ; [bp + 8] - drive number
    mov ah, 0x00 ; "reset"
    mov dl, [bp + 8]
    stc
    int 0x13

    x86_EnterProtectedMode
    [bits 32]

    jc .fail
    xor eax, eax
    jmp .done

.fail:
    mov eax, 1

.done:
    pop edx
    mov esp, ebp
    pop ebp
    ret

;
; uint16_t _cdecl _BIOS_Drive_GetParams(uint8_t driveNumber,
;                                       uint8_t* driveType,
;                                       uint8_t* maxHeadOut,
;                                       uint16_t* maxCylinderOut,
;                                       uint8_t* maxSectorOut);
;
_BIOS_Drive_GetParams:
    [bits 32]
    push ebp
    mov ebp, esp
    push ebx
    push esi

    x86_EnterRealMode
    [bits 16]

    ; [bp + 8] - drive number
    ; [bp + 12] - drive type ptr
    ; [bp + 16] - max head ptr
    ; [bp + 20] - max cylinder ptr
    ; [bp + 24] - max sector ptr

    push es
    push di
    ; ES:DI = 0x0000:0x0000
    xor di, di
    mov es, di
    mov ah, 0x08
    mov dl, [bp + 8]
    stc
    int 0x13
    pop di
    pop es

    x86_EnterProtectedMode
    [bits 32]

    jc .fail

    ; BL = drive type (AT/PS2 floppies only) (see #00242)
    ; CH = low eight bits of maximum cylinder number
    ; CL = maximum sector number (bits 5-0)
    ; high two bits of maximum cylinder number (bits 7-6)
    ; DH = maximum head number
    ; DL = number of drives

    ; Store the drive type
    mov esi, [ebp + 12]
    mov [esi], bl

    ; Store the max head number
    mov esi, [ebp + 16]
    mov [esi], dh

    ; Store the max cylinder number
    mov bl, ch
    mov bh, cl
    shr bh, 6
    mov esi, [ebp + 20]
    mov [esi], bx

    ; Store the max sector number
    and cl, 0b00111111 ; sector number is in bits 5-0
    mov esi, [ebp + 24]
    mov [esi], cl

    xor eax, eax ; success
    jmp .done

.fail:
    mov eax, 1

.done:
    pop esi
    pop ebx
    mov esp, ebp
    pop ebp
    ret

;
; uint16_t _cdecl _BIOS_Drive_ReadSectors(uint8_t driveNumber,
;                                         uint8_t head,
;                                         uint16_t cylinder,
;                                         uint8_t sector,
;                                         uint8_t sectorCount,
;                                         uint8_t far* dataDestination);
;
_BIOS_Drive_ReadSectors:
    [bits 32]
    push ebp
    mov ebp, esp

    ; [bp + 8] - drive number
    ; [bp + 12] - head
    ; [bp + 16] - cylinder
    ; [bp + 20] - sector
    ; [bp + 24] - sector read count
    ; [bp + 28] - data destination

    ; AL = number of sectors to read (must be nonzero)
    ; CH = low eight bits of cylinder number
    ; CL = sector number 1-63 (bits 0-5)
    ; high two bits of cylinder (bits 6-7, hard disk only)
    ; DH = head number
    ; DL = drive number (bit 7 set for hard disk)
    ; ES:BX -> data buffer

    x86_EnterRealMode

    push bx
    push es

    ; Set drive number
    mov dl, [bp + 8]

    ; Set cylinder number
    mov ch, [bp + 16]
    mov cl, [bp + 18]
    shl cl, 6

    ; Set head number
    mov dh, [bp + 12]

    ; Set sector number
    mov al, [bp + 20]
    and al, 0b00111111 ; Clear top bits of sector number
    or cl, al

    ; Set number of sectors to read
    mov al, [bp + 24]

    ; Set destination data buffer
    LinearToSegmentOffset [bp + 28], es, ebx, bx

    ; mov bx, [bp + 16]

    ; mov bl, bh
    ; call ___print_hex
    ; mov bx, [bp + 16]
    ; and bx, 0x0F
    ; call ___print_hex
    ; mov bx, [bp + 16]

    ; mov es, bx
    ; mov bx, [bp + 14]

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

    pop es
    pop bx

    x86_EnterProtectedMode

    jnc .success

.fail:
    mov eax, 1
    jmp .done
.success:
    xor eax, eax ; success
.done:
    mov esp, ebp
    pop ebp
    ret
