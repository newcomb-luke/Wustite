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

	call check_long_mode
	cmp ax, 0
	je .enable_long_mode
	jmp long_mode_failure

	.enable_long_mode:

	cli

	; We will start our PML4T at 0x1000
	; We start by filling it wtih zeros
	mov edi, 0x1000
	mov cr3, edi
	xor eax, eax
	mov ecx, 4096
	rep stosd
	mov edi, 0x1000

	; PML4T -> 0x1000
	; PDPT -> 0x2000
	; PDT -> 0x3000
	; PT -> 0x4000

	mov dword [edi], 0x2003
	add edi, 0x1000
	mov dword [edi], 0x3003
	add edi, 0x1000
	mov dword [edi], 0x4003
	add edi, 0x1000

	mov dword ebx, 3
	mov ecx, 512

	.set_entry:
		mov dword [edi], ebx
		add ebx, 0x1000
		add edi, 8
		loop .set_entry

	; Enable PAE
	mov eax, cr4
	or eax, 1 << 5
	mov cr4, eax

	; Enable long mode
	mov ecx, 0xc0000080
	rdmsr
	or eax, 1 << 8
	wrmsr

	; Enable paging + compatibility mode
	mov eax, cr0
	or eax, 1 << 31
	or eax, 1 << 0
	mov cr0, eax

	; Load the global descriptor table
	lgdt [GDT.pointer]

	; Jump into 64-bit code
	jmp GDT.code:longmode

a20_already:
	mov si, A20_ENABLED_FIRST
	call puts
	jmp start.after_a20

a20_success:
	mov si, A20_SUCCESS
	call puts
	jmp start.after_a20

long_mode_failure:
	mov si, LONG_MODE_UNAVAILABLE
	call puts
	jmp halt
	
a20_failure:
	mov si, A20_FAILURE
	call puts
	jmp halt

halt:
	jmp $

%include "./util.s"
%include "./a20.s"
%include "./long-mode.s"
%include "./gdt.s"

VIDEO_MEM equ 0xb8000
VID_ADDR_REG equ 0x03d4
VID_DATA_REG equ 0x03d5
CUR_LOC_LOW equ 0x0f
CUR_LOC_HIGH equ 0x0e

[bits 64]

longmode:
	cli

	mov edi, VIDEO_MEM
	mov rax, 0x1f201f201f201f20
	mov ecx, 500
	rep stosq

	mov dx, 0x03cc
	in al, dx

	or al, 1 << 0

	mov dx, 0x03c2
	out dx, al

	mov dx, VID_ADDR_REG
	mov al, CUR_LOC_LOW
	out dx, al

	mov dx, VID_DATA_REG
	mov al, 0x01
	out dx, al

	mov dx, VID_ADDR_REG
	mov al, CUR_LOC_HIGH
	out dx, al

	mov dx, VID_DATA_REG
	mov al, 0x00
	out dx, al

	mov rax, 0x1f201f201f541f54
	mov [VIDEO_MEM], rax

	hlt

ALIVE_MSG: db "Reached stage 2", 0x0a, 0x0d, 0
A20_ENABLED_FIRST: db "A20 line was enabled at boot", 0x0a, 0x0d, 0
A20_ENABLING_TRY: db "Attempting to enable A20 line", 0x0a, 0x0d, 0
A20_SUCCESS: db "A20 line successfully enabled", 0x0a, 0x0d, 0
A20_FAILURE: db "ERROR: Failed to enable A20 line", 0x0a, 0x0d, 0
STAGE_3_MSG: db "Reached stage 3", 0x0a, 0x0d, 0
LONG_MODE_SUCCESS: db "Successfully enabled long mode", 0x0a, 0x0d, 0
LONG_MODE_UNAVAILABLE: db "Long mode is not supported on this machine. Cannot boot.", 0x0a, 0x0d, 0
STAGE_4_MSG: db "Reached stage 4", 0x0a, 0x0d, 0

times 512 * 4 db 0
