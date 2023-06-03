[bits 32]

section .text

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
    mov es, ax
    mov fs, ax
    mov gs, ax

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
    mov es, ax
    mov fs, ax
    mov gs, ax
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

    jc .fail_real_mode

.success_real_mode:
    mov word [real_mode_error], 0
    jmp .done_real_mode

.fail_real_mode:
    mov word [real_mode_error], 1

.done_real_mode:

    x86_EnterProtectedMode
    [bits 32]

    mov ax, [real_mode_error]
    and ax, ax
    jnz .fail

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
; extern "cdecl" {
;   _BIOS_Drive_GetParams(drive_number: u8,
;                         buffer: *mut u8) -> u16;
; }
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
    ; [bp + 12] - pointer to buffer

    ; buffer layout:
    ;  u8: drive type
    ;  u8: max head number
    ;  u16: max cylinder number
    ;  u8: max sector number

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

    jc .fail_real_mode

.success_real_mode:
    mov word [real_mode_error], 0
    jmp .done_real_mode

.fail_real_mode:
    mov word [real_mode_error], 1

.done_real_mode:

    x86_EnterProtectedMode
    [bits 32]

    mov ax, [real_mode_error]
    and ax, ax
    jnz .fail

    ; BL = drive type (AT/PS2 floppies only) (see #00242)
    ; CH = low eight bits of maximum cylinder number
    ; CL = maximum sector number (bits 5-0)
    ; high two bits of maximum cylinder number (bits 7-6)
    ; DH = maximum head number
    ; DL = number of drives

    ; Store the drive type
    mov esi, [ebp + 12]
    mov [esi + 0], bl

    ; Store the max head number
    mov esi, [ebp + 12]
    mov [esi + 1], dh

    ; Store the max cylinder number
    mov bl, ch
    mov bh, cl
    shr bh, 6
    mov esi, [ebp + 12]
    mov [esi + 2], bx

    ; Store the max sector number
    and cl, 0b00111111 ; sector number is in bits 5-0
    mov esi, [ebp + 12]
    mov [esi + 4], cl

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

    jc .fail_real_mode

.success_real_mode:
    mov word [real_mode_error], 0
    jmp .done_real_mode

.fail_real_mode:
    mov word [real_mode_error], 1

.done_real_mode:

    x86_EnterProtectedMode

    mov ax, [real_mode_error]
    and ax, ax
    jnz .fail

    xor eax, eax ; success
    jmp .done

.fail:
    mov eax, 1
.done:
    mov esp, ebp
    pop ebp
    ret

real_mode_error: dw 0
