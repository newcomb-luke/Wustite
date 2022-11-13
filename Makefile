final: loader bootloader
	cat bootloader/build/bootloader.bin loader.bin > final.bin

bootloader: bootloader/src/bootloader.asm bootloader/src/bios-utils.asm
	mkdir -p bootloader/build/
	nasm -fbin bootloader/src/bootloader.asm -o bootloader/build/bootloader.bin

loader: loader/src/main.rs
	cd loader; \
	cargo xbuild --release --target target.json; \
	objcopy -I elf64-x86-64 -O binary target/target/release/loader ../loader.bin

clean:
	rm -dr bootloader/build/
	rm final.bin
	rm loader.bin
	cd loader; \
	cargo clean

run: final
	qemu-system-x86_64 final.bin
