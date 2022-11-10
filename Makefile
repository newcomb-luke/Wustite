bootloader.bin: bootloader.s
	nasm -fbin bootloader.s -o bootloader.bin

clean:
	rm bootloader.bin

run:
	qemu-system-x86_64 bootloader.bin
