[bits 16]

section .entry

global entry
extern _start
extern __bss_start
extern __bss_end

entry:
	; the boot drive number is in dl
    mov [g_BootDrive], dl

    xor ax, ax
    mov ds, ax
    mov es, ax
    mov ss, ax
    mov sp, 0xFFF0
	mov bp, sp

    ; Disable interrupts
    cli

    ; Enable the A20 line
    call __enable_a20
    cmp ax, 0
    jne halt

    ; Load the GDT
    call load_GDT

    ; Set protected mode bit in control register 0
    mov eax, cr0
    or al, 1
    mov cr0, eax

    jmp dword 0x08:pmode32

halt:
    cli
    hlt
    jmp $

[bits 32]
pmode32:
    ; Initialize segment registers
    mov ax, 0x10 ; Offset 16 into the GDT
    mov ds, ax
    mov ss, ax

    ; Zero the .bss section
    mov edi, __bss_start
    mov ecx, __bss_end
    sub ecx, edi
    mov al, 0
    cld
    rep stosb

    xor edx, edx
    mov dl, [g_BootDrive]
    push edx
	call _start

	jmp $

[bits 16]
load_GDT:
    lgdt [GDT_Descriptor]
    ret

[bits 16]
GDT:
    ; Null descriptor
    dq 0
    ; 32-bit code segment
    ; Here we want a segment that contains all of memory
    ; Base = 0x00000000
    ; Limit = 0xFFFFFFFF
    dw 0xFFFF                  ; limit (bits 0-15)
    dw 0x0000                  ; base (bits 0-15) 0x0000 to start at the beginning of memory
    db 0x00                    ; base (bits 16-23) 0x00
    db 0b10011010              ; All sorts of flags:
                               ; Present, Ring 0, Non-system segment, Code/Data, Code, Readable, Nonconforming
    db 0b11001111              ; Granularity 4KiB, 32 bit, non-64 bit, and limit (bits 16-19) all 0xFF
    db 0x00                    ; base (bits 24-31) 0x00
    ; 32-bit data segment
    ; Here we want a segment that contains all of memory
    ; Base = 0x00000000
    ; Limit = 0xFFFFFFFF
    dw 0xFFFF                  ; limit (bits 0-15)
    dw 0x0000                  ; base (bits 0-15) 0x0000 to start at the beginning of memory
    db 0x00                    ; base (bits 16-23) 0x00
    db 0b10010010              ; All sorts of flags:
                               ; Present, Ring 0, Non-system segment, Code/Data, Data, Writable, Expand up
    db 0b11001111              ; Granularity 4KiB, 32 bit, non-64 bit, and limit (bits 16-19) all 0xFF
    db 0x00                    ; base (bits 24-31) 0x00
    ; 16-bit code segment
    ; Here we want a segment that contains all of memory
    ; Base = 0x00000000
    ; Limit = 0xFFFFFFFF
    dw 0xFFFF                  ; limit (bits 0-15)
    dw 0x0000                  ; base (bits 0-15) 0x0000 to start at the beginning of memory
    db 0x00                    ; base (bits 16-23) 0x00
    db 0b10011010              ; All sorts of flags:
                               ; Present, Ring 0, Non-system segment, Code/Data, Code, Readable, Nonconforming
    db 0b00001111              ; Granularity 1 byte, 16 bit, non-64 bit, and limit (bits 16-19) all 0xFF
    db 0x00                    ; base (bits 24-31) 0x00
    ; 16-bit data segment
    ; Here we want a segment that contains all of memory
    ; Base = 0x00000000
    ; Limit = 0xFFFFFFFF
    dw 0xFFFF                  ; limit (bits 0-15)
    dw 0x0000                  ; base (bits 0-15) 0x0000 to start at the beginning of memory
    db 0x00                    ; base (bits 16-23) 0x00
    db 0b10010010              ; All sorts of flags:
                               ; Present, Ring 0, Non-system segment, Code/Data, Data, Writable, Expand up
    db 0b00001111              ; Granularity 1 byte, 16 bit, non-64 bit, and limit (bits 16-19) all 0xFF
    db 0x00                    ; base (bits 24-31) 0x00

[bits 16]
GDT_Descriptor:
    dw GDT_Descriptor - GDT - 1 ; Size of GDT in bytes - 1
    dd GDT                      ; Offset of GDT

g_BootDrive: db 0

%include "a20.asm"
