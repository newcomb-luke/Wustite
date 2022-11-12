bootloader: stage1.bin stage2.bin
	cat stage1.bin stage2.bin > bootloader.bin

stage1.bin: stage1.s stage1-util.s
	nasm -fbin stage1.s -o stage1.bin

stage2.bin: stage2.s stage1-util.s util.s a20.s long-mode.s gdt.s
	nasm -fbin stage2.s -o stage2.bin

clean:
	rm stage1.bin
	rm stage2.bin
	rm bootloader.bin

run: bootloader
	qemu-system-x86_64 bootloader.bin
