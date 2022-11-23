[bits 32]

KERNEL_STACK_START equ 0x00200000

global long_mode_jump
long_mode_jump:
    [bits 32]
    push ebp
    mov ebp, esp

    ; Juuuust in case
    cli

    ; [ebp + 8] - 32-bit pointer to the kernel entry point

    ; Quickly save the entry point
    mov edx, [ebp + 8]
    mov [ENTRY_POINT], edx

    ; Tasks according to the Intel Developer's Guide:
    ; 1. Disable paging
    ; 2. Enable PAE
    ; 3. Load the page table
    ; 4. Set Long Mode Enable bit in EFER
    ; (Now in compatability mode)
    ; 5. Enable paging (will enable long mode)

    ; 1. The bootloader never enables paging, so already done!

    ; 2. Enable PAE
    mov eax, cr4
    or eax, 1 << 5
    mov cr4, eax

    ; 3. Load the page table
    ; The page table is already set up in main.c!
    ; We just have to load it
    ; I need to find a better way to do this, but currently in memory.h
    ; the page tables start at 0x00010000, so load that:
    mov edi, 0x00010000
    mov cr3, edi

    ; 64-bit TSS

    ; 4. Enable long mode
    mov ecx, 0xc0000080
    rdmsr
    or eax, 1 << 8
    wrmsr

    ; We are now in 32-bit compatability mode!

    ; Enable paging
    mov eax, cr0
    or eax, 1 << 31
    mov cr0, eax

    ; We are now in 64-bit mode!
    ; We have to jump into a 64-bit code segment
    ; before everything really applies

    ; Load 64-bit GDT entries
    lgdt [L_GDT_Descriptor]

    ; Finally, jump into long mode!
    jmp L_GDT_CODE:in_longmode

in_longmode:
    [bits 64]
    cli

    ; Set up segments
    mov ax, L_GDT_DATA
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    mov ss, ax

    ; Set up the stack, quickly!
    mov rsp, KERNEL_STACK_START
    mov rbp, rsp

    ; Load the entry point into rdx
    ; Sign-extend upper bits of rdx for a 32-bit address
    xor rdx, rdx
    not rdx
    mov edx, [ENTRY_POINT]

    ; Jump to the entry point!
    call rdx

    ; Never return
    cli
    hlt
    jmp $

[bits 32]

ENTRY_POINT: dd 0

L_GDT:
    L_GDT_NULL equ $ - L_GDT
    ; Null descriptor
    dq 0
    L_GDT_CODE equ $ - L_GDT
    ; 64-bit code segment
    ; Almost all fields are ignored, but just for a little bit of sanity, we will
    ; set it up like a 32-bit code segment for the most part
    dw 0xFFFF                            ; Limit (bits 0-15)
    dw 0x0000                            ; Base (bits 0-15) 0x0000 to start at the beginning of memory
    db 0x00                              ; Base (bits 16-23) 0x00
    ;    v-- Present bit
    ;    | v-- DPL - ring 0
    ;    | |
    ;    | | v-- Always set to 11 for code segments
    ;    | | |v-- "Nonconforming"
    ;    | | ||v-- Readable
    ;    | | |||v-- Not accessed
    db 0b10011010
    ;    v-- Granularity 4KiB
    ;    |v-- 64-bit operands
    ;    ||v-- 64 bit!
    ;    |||v-- "Available to software"
    ;    ||||v-- Limit (bits 16-19) 0xFF
    db 0b10111111
    db 0x00                               ; Based (bits 24-31) 0x00
    L_GDT_DATA equ $ - L_GDT
    ; 64-bit data segment
    ; REALLY all of the fields are ignored, like all bit a single bit
    ; We still set it up like a 32-bit segment though...
    dw 0xFFFF
    dw 0x0000
    db 0x00
    ;    v-- Present bit
    ;    | v-- DPL - ring 0
    ;    | |
    ;    | | v-- Always set to 10 for data segments
    ;    | | |v-- Expand down (not set, so expand up)
    ;    | | ||v-- Writable
    ;    | | |||v-- Not accessed
    db 0b10010010
    ;    v-- Granularity 4KiB
    ;    |v-- 32-bit operands I guess, 64 bit isn't listed
    ;    ||v-- Reserved?
    ;    |||v-- "Available to software"
    ;    ||||v-- Limit (bits 16-19) 0xFF
    db 0b11010000
    db 0x00
    L_GDT_TSS equ $ - L_GDT
    ; 64-bit Task State Segment
    ; Slightly different than the other segments, it is twice as large
    ; All 0's base address 15:0, and all 1's segment limit 15:0
    dd 0x0000FFFF
    ;    v======v-- Base address 31:24
    ;    |      |v-- Granularity 4KiB
    ;    |      ||v=v-- Ignored/available to software
    ;    |      ||| |v==v-- Segment limit 19:16
    ;    |      ||| ||  |v-- Present bit
    ;    |      ||| ||  ||vv-- Ring 0
    ;    |      ||| ||  || |v-- Always 0
    ;    |      ||| ||  || ||v==v-- Type: 1001 indicates an "available" 64-bit TSS
    ;    |      ||| ||  | |||  |v======v-- Base address 23:16
    ;    |      ||| ||  || v||  ||      |
    dd 0b00000000100011111000100100000000
    ; Base address 63:32 of all 0's
    dd 0x00000000
    ; Reserved/Ignored/Must be set to 0, it all just works
    dd 0x00000000


L_GDT_Descriptor:
    dw L_GDT_Descriptor - L_GDT - 1      ; Size of GDT in bytes - 1
    dq L_GDT                             ; Offset of GDT in memory

; returns 0 in ax if we successfully switched to long mode,
; 1 otherwise
check_long_mode:
    [bits 32]
	pusha

	call is_cpuid_available
	cmp eax, 1
	jne .no

	call is_extended_cpuid_available
	cmp eax, 1
	jne .no

	; If the 29th bit it set, we have long mode
	mov eax, 0x80000001
	cpuid
	test edx, 1 << 29
	jz .no

.yes:
    popa
    xor ax, ax
    ret

.no:
    popa
    mov ax, 1
    ret

; returns 1 in ax if extended cpuid is available, 0 if not
global is_extended_cpuid_available
is_extended_cpuid_available:
    [bits 32]
	push ebp
	mov ebp, esp
	pusha

	mov eax, 0x80000000
	cpuid
	cmp eax, 0x80000001
	jb .no

	popa
	mov eax, 1
	mov esp, ebp
	pop ebp
	ret

.no:
	popa
	xor eax, eax
	mov esp, ebp
	pop ebp
	ret


; returns 1 in ax if cpuid is available, 0 if not
global is_cpuid_available
is_cpuid_available:
    [bits 32]
	push ebp
	mov ebp, esp
	pusha

	pushfd
	pop eax
	mov ecx, eax

	; Flip 'ID' bit to 1
	xor eax, 0x200000

	push eax
	popfd

	pushfd
	pop eax

	; These should not be the same, if they are
	; then our change didn't stick: We don't have
	; cpuid support.
	cmp ecx, eax
	je .no

	popa
	mov eax, 1
	mov esp, ebp
	pop ebp
	ret

.no:
	popa
	xor eax, eax
	mov esp, ebp
	pop ebp
	ret