[org 0x7c00 + 512]
[bits 16]

section .text

global start

start:
	mov si, ALIVE_MSG
	call puts

	call test_a20
	cmp ax, 0
	je a20_already

	mov si, A20_ENABLING_TRY
	call puts

	call try_enable_a20
	cmp ax, 0
	je a20_success
	jmp a20_failure
	
	.after_a20:
	mov si, STAGE_3_MSG
	call puts

	jmp halt

a20_already:
	mov si, A20_ENABLED_FIRST
	call puts
	jmp start.after_a20

a20_success:
	mov si, A20_SUCCESS
	call puts
	jmp start.after_a20
	
a20_failure:
	mov si, A20_FAILURE
	call puts
	jmp halt

halt:
	jmp $

%include "./util.s"
%include "./a20.s"

ALIVE_MSG: db "Reached stage 2", 0x0a, 0x0d, 0
A20_ENABLED_FIRST: db "A20 line was enabled at boot", 0x0a, 0x0d, 0
A20_ENABLING_TRY: db "Attempting to enable A20 line", 0x0a, 0x0d, 0
A20_SUCCESS: db "A20 line successfully enabled", 0x0a, 0x0d, 0
A20_FAILURE: db "ERROR: Failed to enable A20 line", 0x0a, 0x0d, 0
STAGE_3_MSG: db "Reached stage 3", 0x0a, 0x0d, 0

times 512 * 4 db 0
