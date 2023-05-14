.PHONY: all floppy_image boot_sector bootloader clean always kernel

all: boot_components floppy_image kernel

TARGET_ASM=nasm

BUILD_DIR=$(abspath build)
KERNEL_BASE_DIR=$(abspath kernel)

#
# Boot components
#

boot_components: boot_sector bootloader

boot_sector: $(BUILD_DIR)/boot_sector.bin

$(BUILD_DIR)/boot_sector.bin: always
	nasm -fbin boot-sector/boot-sector.asm -o $(BUILD_DIR)/boot-sector.bin

bootloader: $(BUILD_DIR)/bootloader.bin

$(BUILD_DIR)/bootloader.bin: always FORCE
	nasm -f elf bootloader/src/entry.asm -o target/i686-none-eabi/entry.o
	nasm -f elf bootloader/src/bios.asm -o target/i686-none-eabi/bios.o
	ar rcs bootloader/libentry.a target/i686-none-eabi/entry.o
	ar rcs bootloader/libbios.a target/i686-none-eabi/bios.o
	cargo build --release -Z build-std=core --target=i686-none-eabi.json --package=bootloader
	objcopy -I elf32-i386 -O binary target/i686-none-eabi/release/bootloader $(BUILD_DIR)/bootloader.bin

#
# Floppy image
#

floppy_image: $(BUILD_DIR)/boot_floppy.img

$(BUILD_DIR)/boot_floppy.img: boot_sector bootloader kernel
	dd if=/dev/zero of=$(BUILD_DIR)/boot_floppy.img bs=512 count=2880
	mkfs.fat -F 12 -n "WUSTITE1" $(BUILD_DIR)/boot_floppy.img
	dd if=$(BUILD_DIR)/boot-sector.bin of=$(BUILD_DIR)/boot_floppy.img conv=notrunc
	mcopy -i $(BUILD_DIR)/boot_floppy.img $(BUILD_DIR)/kernel.o "::kernel.o"
	mcopy -i $(BUILD_DIR)/boot_floppy.img test.txt "::test.txt"
	mcopy -i $(BUILD_DIR)/boot_floppy.img $(BUILD_DIR)/bootloader.bin "::boot.bin"

# 
# Kernel
#
kernel: $(BUILD_DIR)/kernel.o

$(BUILD_DIR)/kernel.o: always FORCE
	RUSTFLAGS="-C code-model=kernel -C relocation-model=static" cargo build --release -Z build-std=core --target=x86_64-none-eabi.json --package=kernel
	cp target/x86_64-none-eabi/release/kernel $(BUILD_DIR)/kernel.o

FORCE: ;

#
# Always
#
always:
	mkdir -p $(BUILD_DIR)

# 
# Clean
#
clean:
	cargo clean
	rm -rf build

run: $(BUILD_DIR)/boot_floppy.img
	qemu-system-x86_64 -m 2G -fda $(BUILD_DIR)/boot_floppy.img

debug: $(BUILD_DIR)/boot_floppy.img
	bochs -f bochs.cfg -q
