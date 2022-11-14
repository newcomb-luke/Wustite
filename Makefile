ASM=nasm
SRC_DIR=src
BUILD_DIR=build
BOOTLOADER_SRC_DIR=bootloader/src

.PHONY: all floppy_image bootloader clean always

all: bootloader floppy_image

#
# Bootloader
#

bootloader: $(BUILD_DIR)/bootloader.bin

$(BUILD_DIR)/bootloader.bin: $(BOOTLOADER_SRC_DIR)/bootloader.asm $(BOOTLOADER_SRC_DIR)/bios-utils.asm always
	$(ASM) -fbin $(BOOTLOADER_SRC_DIR)/bootloader.asm -o $(BUILD_DIR)/bootloader.bin

#
# Floppy image
#

floppy_image: $(BUILD_DIR)/boot_floppy.img

$(BUILD_DIR)/boot_floppy.img: bootloader kernel
	dd if=/dev/zero of=$(BUILD_DIR)/boot_floppy.img bs=512 count=2880
	mkfs.fat -F 12 -n "WUSTITE1" $(BUILD_DIR)/boot_floppy.img
	dd if=$(BUILD_DIR)/bootloader.bin of=$(BUILD_DIR)/boot_floppy.img conv=notrunc
	mcopy -i $(BUILD_DIR)/boot_floppy.img $(BUILD_DIR)/kernel.bin "::kernel.bin"

# 
# Kernel
#
kernel: $(BUILD_DIR)/kernel.bin

$(BUILD_DIR)/kernel.bin: always
	dd if=/dev/zero of=$(BUILD_DIR)/kernel.bin bs=512 count=1

#
# Always
#
always:
	mkdir -p $(BUILD_DIR)

# 
# Clean
#
clean:
	rm -rf $(BUILD_DIR)/*

run: $(BUILD_DIR)/boot_floppy.img
	qemu-system-x86_64 -fda $(BUILD_DIR)/boot_floppy.img
