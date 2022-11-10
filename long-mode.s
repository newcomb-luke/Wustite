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
