[bits 16]

section .entry

global entry
extern bootloader_entry
extern __bss_start
extern __bss_end

entry:
    xor ax, ax
    mov ds, ax
    mov es, ax
    mov ss, ax
    mov sp, 0x7b00
    mov bp, sp

    ; Enable the A20 line
    call __enable_a20
    cmp ax, 0
    jne halt

    ; Disable interrupts
    cli

    ; Load the GDT
    lgdt [GDT_Descriptor]

    ; Set protected mode bit in control register 0
    mov eax, cr0
    or al, 1
    mov cr0, eax

    jmp 0x08:pmode32

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

    ; Re-initialize the stack
    mov esp, 0x7b00
    mov ebp, esp

    ; Zero the .bss section
    mov edi, __bss_start
    mov ecx, __bss_end
    sub ecx, edi
    mov al, 0
    cld
    rep stosb

    sti

	call bootloader_entry

	jmp $

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
    dw 40 - 1 ; Size of GDT in bytes - 1
    dd GDT                      ; Offset of GDT

KEYBOARD_DATA_PORT           equ 0x60
KEYBOARD_STATUS_PORT         equ 0x64 ; If this port is being read from, it is the status register
KEYBOARD_CMD_PORT            equ 0x64 ; If this port is being written to, it is the command register

CONTROLLER_DISABLE_KEYBOARD  equ 0xAD
CONTROLLER_ENABLE_KEYBOARD   equ 0xAE
CONTROLLER_READ_OUTPUT_PORT  equ 0xD1
CONTROLLER_WRITE_OUTPUT_PORT equ 0xD1

OUTPUT_BUFFER_STATUS_BIT     equ 1
INPUT_BUFFER_STATUS_BIT      equ 2

A20_GATE_OUTPUT_PORT_BIT     equ 2

__enable_a20:
	pusha

    ; Is A20 already enabled?
    call test_a20
    cmp ax, 0
    je .success

    ; BIOS method, uses a BIOS interrupt to enable a20
    mov ax, 0x2401
    int 0x15

    call test_a20
    cmp ax, 0
    je .success

    ; Now try the keyboard controller method
    call try_enable_a20_keyboard

    call test_a20
    cmp ax, 0
    je .success

    ; Last resort "Fast A20" method
    in al, 0x92
    or al, 2
    out 0x92, al

    call test_a20
    cmp ax, 0
    je .success

    jmp .failure

.success:
    popa
    xor ax, ax
    jmp .finally
.failure:
    popa
    mov ax, 0x01
.finally:
    ret

try_enable_a20_keyboard:
    push ax

    ; Disable keyboard
    call wait_controller_ready_for_input
    mov al, CONTROLLER_DISABLE_KEYBOARD
    out KEYBOARD_CMD_PORT, al

    ; Read the control output port
    call wait_controller_ready_for_input
    mov al, CONTROLLER_READ_OUTPUT_PORT
    out KEYBOARD_CMD_PORT, al
    ; Wait for the data to be ready
    call wait_controller_data_ready
    in al, KEYBOARD_DATA_PORT
    ; Push the value of the control output port to the stack.
    push ax

    ; Write to the control output port
    call wait_controller_ready_for_input
    mov al, CONTROLLER_WRITE_OUTPUT_PORT
    out KEYBOARD_CMD_PORT, al
    call wait_controller_ready_for_input
    ; Get the value of the control output port
    pop ax
    ; Set the A20 address line enable bit
    or al, A20_GATE_OUTPUT_PORT_BIT
    ; Write the value
    out KEYBOARD_DATA_PORT, al

    ; Re-enable keyboard
    call wait_controller_ready_for_input
    mov al, CONTROLLER_ENABLE_KEYBOARD
    out KEYBOARD_CMD_PORT, al

    ; Wait for the chip to be done
    call wait_controller_ready_for_input

    pop ax
    ret

; return value is stored in ax
; 	if ax is 0, the a20 line is enabled
; 	if ax is 1, the a20 line is disabled
test_a20:
	push ds
    push bx
    push dx

    ; Set data segment register to 0x0000
    xor ax, ax
    mov ds, ax
    ; 0x0000:0x7DFE is the address of the boot sector magic number 0xAA55
    mov bx, 0x7DFE
    ; Re-write it juuuuuust in case
    mov ax, 0xAA55
    mov [ds:bx], ax
    ; Now read it
    mov ax, [ds:bx]
    ; Store the read value
	push ax

	; Set the data segment register to 0xffff
	mov ax, 0xffff
	mov ds, ax
	; Get the previously read value
	pop ax

	; After we use 0xffff as our segment offset, our
	; new address for the magic number is 0x7E0E
	mov bx, 0x7E0E

    ; Read 0xffff:0x7E0E
	mov dx, [ds:bx]

	; restore ds to 0x0000
	push ax
	xor ax, ax
	mov ds, ax
	pop ax

    ; If the read value before and after are not equal, then we have the a20 line enabled
	cmp ax, dx
	jne .enabled

	; The OSdev wiki recommends using a different offset and seeing if it is still
	; the same, because it could have just been a fluke if we didn't see it the first time

    ; Set data segment register to 0x0000
    xor ax, ax
    mov ds, ax
    ; 0x0000:0x7DFF is some random value
    mov bx, 0x7DFF
    mov ax, [ds:bx]
    ; Store the read value
	push ax

	; Set the data segment register to 0xffff
	mov ax, 0xffff
	mov ds, ax
	; Get the previously read value
	pop ax

	; After we use 0xffff as our segment offset, our
	; new address for whatever data we previously accessed is 0x7E0F
	mov bx, 0x7E0F

    ; Read 0xffff:0x7E0F
	mov dx, [ds:bx]

	; restore ds to 0x0000
	push ax
	xor ax, ax
	mov ds, ax
	pop ax

	cmp ax, dx
	jne .enabled

.disabled:
    pop dx
    pop bx
    pop ds
    mov ax, 1
    ret

.enabled:
    pop dx
    pop bx
    pop ds
    xor ax, ax
    ret

; Waits until the input buffer is ready for us to write to
wait_controller_ready_for_input:
	push ax

	in al, KEYBOARD_STATUS_PORT
	test al, INPUT_BUFFER_STATUS_BIT
	jnz wait_controller_ready_for_input

	pop ax
	ret

; Waits until the output buffer is ready for us to read a result from
wait_controller_data_ready:
	push ax

	in al, KEYBOARD_STATUS_PORT
	test al, OUTPUT_BUFFER_STATUS_BIT
	jz wait_controller_data_ready

	pop ax
	ret
