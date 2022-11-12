bootloader: bootloader/src/bootloader.asm bootloader/src/bios-utils.asm
	mkdir -p bootloader/build/
	nasm -fbin bootloader/src/bootloader.asm -o bootloader/build/bootloader.bin

clean:
	rm -dr bootloader/build/

run: bootloader
	qemu-system-x86_64 bootloader/build/bootloader.bin
