final: secondary
	cat bootloader/build/bootloader.bin secondary.bin > final.bin

bootloader: bootloader/src/bootloader.asm bootloader/src/bios-utils.asm
	mkdir -p bootloader/build/
	nasm -fbin bootloader/src/bootloader.asm -o bootloader/build/bootloader.bin

secondary: secondary.asm bootloader
	nasm -fbin secondary.asm -o secondary.bin

clean:
	rm -dr bootloader/build/
	rm final.bin
	rm secondary.bin

run: final
	qemu-system-x86_64 final.bin
