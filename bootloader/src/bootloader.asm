[org 0x7c00]
[bits 16]

_start:
    ; Disable interrupts
    cli

    ; Zero the segment registers
    jmp 0x0000:zero_seg
    zero_seg:
        xor ax, ax
        mov ss, ax
        mov ds, ax
        mov es, ax
        mov fs, ax
        mov gs, ax
        ; Put the stack in the top section above where we are loaded
        mov sp, 0x7bff
        cld

    ; Resume interrupts
    sti

    ; Clear the screen
    mov ah, 0x00
    ; text mode 80x25 16 colors
    mov al, 0x03
    int 0x10

    ; Reset the disk controller, just in case
    push ax
    xor ax, ax ; 0 = reset
    mov dl, 0x80 ; Hard drive
    int 0x13
    pop ax

    mov si, STARTUP_MSG
    call b_puts

    mov al, 0x03 ; Number of sectors to read, in this case 3
    mov cl, 0x02 ; We are in the first sector, the next one is sector 2
    mov bx, 0x7c00 + 512 ; The location right after the boot sector
    mov si, DISK_READ_ERROR_MSG
    call b_read_disk

    jmp bx

%include "./bootloader/src/bios-utils.asm"

STARTUP_MSG: db "Bootloader start", 0x0a, 0x0d, 0
DISK_READ_ERROR_MSG: db "Error: Could not load sector from disk", 0x0a, 0x0d, 0

; Padding
times 510 - ($ - $$) db 0
; Magic number to declare a boot sector
dw 0xaa55

_stage2:
    mov si, SECTOR_2_MSG
    call b_puts

    call test_a20
    cmp ax, 0
    je a20_already

    mov si, A20_ENABLING_TRY
    call b_puts

    call try_enable_a20
    cmp ax, 0
    je a20_success
    jmp a20_failure

    .after_a20:
    mov si, LONG_MODE_TRY
    call b_puts

    call check_long_mode
    cmp ax, 0

    je .enable_long_mode
    jmp long_mode_failure

    .enable_long_mode:
    mov eax, longmode
    call enable_long_mode

    jmp halt

a20_already:
    mov si, A20_ENABLED_ALREADY
    call b_puts
    jmp _stage2.after_a20

a20_success:
    mov si, A20_SUCCESS
    call b_puts
    jmp _stage2.after_a20

a20_failure:
    mov si, A20_FAILURE
    call b_puts
    jmp halt

long_mode_failure:
    mov si, LONG_MODE_UNAVAILABLE
    call b_puts
    jmp halt

halt:
    jmp $

%include "./bootloader/src/stage2-bios-utils.asm"
%include "./bootloader/src/a20.asm"
%include "./bootloader/src/gdt.asm"
%include "./bootloader/src/long-mode.asm"

SECTOR_2_MSG: db "Reached sector 2", 0x0a, 0x0d, 0
A20_ENABLED_ALREADY: db "A20 line was enabled at boot", 0x0a, 0x0d, 0
A20_ENABLING_TRY: db "Attempting to enable A20 line", 0x0a, 0x0d, 0
A20_SUCCESS: db "A20 line successfully enabled", 0x0a, 0x0d, 0
A20_FAILURE: db "ERROR: Failed to enable A20 line", 0x0a, 0x0d, 0
LONG_MODE_TRY: db "Attempting to enable long mode", 0x0a, 0x0d, 0
LONG_MODE_SUCCESS: db "Successfully enabled long mode", 0x0a, 0x0d, 0
LONG_MODE_UNAVAILABLE: db "Long mode is not supported on this machine. Cannot boot.", 0x0a, 0x0d, 0

[bits 64]

longmode:
	cli

	jmp 0x7c00 + 512 * 3

	hlt

times (512 * 2) - (($ - $$) - 512) db 0