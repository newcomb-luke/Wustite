GDT:
	.null: equ $ - GDT
		dw 0
		dw 0
		db 0
		db 0
		db 0
		db 0
	
	.code: equ $ - GDT
		dw 0
		dw 0
		db 0
		db 10011000b
		db 00100000b
		db 0
	
	.data: equ $ - GDT
		dw 0
		dw 0
		db 0
		db 10000000b
		db 0
		db 0
	
	.pointer:
		dw $ - GDT - 1
		dq GDT

