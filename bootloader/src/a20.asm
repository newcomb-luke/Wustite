; return value is stored in ax
; 	if ax is 0, the a20 line is enabled
; 	if ax is 1, the a20 line is disabled
test_a20:
	pusha
	cli

	mov ax, [0x7dfe]
	mov dx, ax
	; call print_hex

	push ax
	mov ax, 0xffff
	mov ds, ax
	pop ax

	; After we use 0xffff as our segment offset, our
	; new address for the magic number is 0x7e0e
	mov bx, 0x7e0e

	mov dx, [ds:bx]

	; restore dx to 0
	push ax
	xor ax, ax
	mov ds, ax
	pop ax

	; call print_hex

	cmp ax, dx
	jne .enabled

	mov ax, [0x7dff]
	mov dx, ax

	; call print_hex

	push ax
	mov ax, 0xffff
	mov ds, ax
	pop ax

	mov bx, 0x7e0f

	mov dx, [ds:bx]

	; restore dx to 0
	push ax
	xor ax, ax
	mov ds, ax
	pop ax

	; call print_hex

	cmp ax, dx
	jne .enabled

	.disabled:
		sti
		popa
		mov ax, 1
		ret

	.enabled:
		sti
		popa
		xor ax, ax
		ret

; return value is stored in ax
; 	if ax is 0, the a20 line was enabled
; 	if ax is 1, the a20 line could not be enabled
try_enable_a20:
	pusha

	; BIOS method
	mov ax, 0x2401
	int 0x15

	call test_a20
	cmp ax, 0
	je .success

	; Keyboard controller method
	cli
	call wait_controller_ready
	mov al, 0xad ; Disable keyboard
	out 0x64, al

	call wait_controller_ready
	mov al, 0xd0 ; Read from input
	out 0x64, al

	call wait_controller_data
	in al, 0x60
	push ax

	call wait_controller_ready
	mov al, 0xd1 ; Write to output
	out 0x64, al

	call wait_controller_ready
	pop ax
	or al, 2
	out 0x60, al

	call wait_controller_ready
	mov al, 0xae ; Enable keyboard
	out 0x64, al

	call wait_controller_ready
	sti
	; End keyboard controller method

	call test_a20
	cmp ax, 0
	je .success

	; Fast A20 method
	in al, 0x92
	or al, 2
	out 0x92, al

	call test_a20
	cmp ax, 0
	je .success

	.failure:
		popa
		mov ax, 1
		ret

	.success:
		popa
		xor ax, ax
		ret

wait_controller_ready:
	push ax

	in al, 0x64
	test al, 2
	jnz wait_controller_ready

	pop ax
	ret

wait_controller_data:
	push ax

	in al, 0x64
	test al, 1
	jz wait_controller_data

	pop ax
	ret
