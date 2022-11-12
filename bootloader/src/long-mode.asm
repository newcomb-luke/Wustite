; returns 0 in ax if we successfully switched to long mode,
; 1 otherwise
check_long_mode:
	pusha

	call is_cpuid_available
	cmp ax, 0
	jne .no

	call is_extended_cpuid_available
	cmp ax, 0
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

; returns 0 in ax if extended cpuid is available, 1 if not
is_extended_cpuid_available:
	pusha

	mov eax, 0x80000000
	cpuid
	cmp eax, 0x80000001
	jb .no

	popa
	xor ax, ax
	ret

	.no:
	popa
	mov ax, 1
	ret


; returns 0 in ax if cpuid is available, 1 if not
is_cpuid_available:
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
	xor ax, ax
	ret

	.no:
	popa
	mov ax, 1
	ret

; This function cannot preserve any registers, so don't expect it to
enable_long_mode:
    push eax
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

	jmp GDT.code:longmode
