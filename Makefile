.PHONY: all floppy_image boot_sector bootloader clean always kernel

all: boot_components floppy_image

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

$(BUILD_DIR)/bootloader.bin: always
	cargo build -Z build-std=core --target=i686-none-eabi.json --package=bootloader
	objcopy -I elf32-i386 -O binary target/i686-none-eabi/debug/bootloader $(BUILD_DIR)/bootloader.bin

#
# Floppy image
#

floppy_image: $(BUILD_DIR)/boot_floppy.img

$(BUILD_DIR)/boot_floppy.img: boot_sector bootloader
	dd if=/dev/zero of=$(BUILD_DIR)/boot_floppy.img bs=512 count=2880
	mkfs.fat -F 12 -n "WUSTITE1" $(BUILD_DIR)/boot_floppy.img
	dd if=$(BUILD_DIR)/boot-sector.bin of=$(BUILD_DIR)/boot_floppy.img conv=notrunc
	mcopy -i $(BUILD_DIR)/boot_floppy.img $(BUILD_DIR)/bootloader.bin "::boot.bin"
	# mcopy -i $(BUILD_DIR)/boot_floppy.img $(BUILD_DIR)/kernel.o "::kernel.o"
	# mcopy -i $(BUILD_DIR)/boot_floppy.img test.txt "::test.txt"

# 
# Kernel
#
kernel: $(BUILD_DIR)/kernel.o $(KERNEL_BASE_DIR)/link.x $(KERNEL_BASE_DIR)/target.json

$(BUILD_DIR)/kernel.o: always FORCE
	cd kernel; \
	cargo xbuild --target target.json
	cp kernel/target/target/release/kernel $(BUILD_DIR)/kernel.o
	# cargo xbuild --release --target target.json
	# cp kernel/target/target/release/kernel $(BUILD_DIR)/kernel.o

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
