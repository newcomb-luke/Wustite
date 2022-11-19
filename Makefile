.PHONY: all floppy_image bootloader secondary clean always kernel

all: bootloader floppy_image

include toolchain/config.mk

include toolchain/toolchain.mk

#
# Bootloader
#

bootloader: primary secondary

primary: $(BUILD_DIR)/primary.bin

$(BUILD_DIR)/primary.bin: always
	$(MAKE) -C $(SRC_DIR)/bootloader/primary BUILD_DIR=$(abspath $(BUILD_DIR))

secondary: $(BUILD_DIR)/secondary.bin

$(BUILD_DIR)/secondary.bin: always
	$(MAKE) -C $(SRC_DIR)/bootloader/secondary BUILD_DIR=$(abspath $(BUILD_DIR))

#
# Floppy image
#

floppy_image: $(BUILD_DIR)/boot_floppy.img

$(BUILD_DIR)/boot_floppy.img: bootloader secondary kernel
	dd if=/dev/zero of=$(BUILD_DIR)/boot_floppy.img bs=512 count=2880
	mkfs.fat -F 12 -n "WUSTITE1" $(BUILD_DIR)/boot_floppy.img
	dd if=$(BUILD_DIR)/primary.bin of=$(BUILD_DIR)/boot_floppy.img conv=notrunc
	mcopy -i $(BUILD_DIR)/boot_floppy.img $(BUILD_DIR)/secondary.bin "::secboot.bin"
	mcopy -i $(BUILD_DIR)/boot_floppy.img $(BUILD_DIR)/kernel.o "::kernel.o"
	mcopy -i $(BUILD_DIR)/boot_floppy.img test.txt "::test.txt"

# 
# Kernel
#
kernel: $(BUILD_DIR)/kernel.o

$(BUILD_DIR)/kernel.o: always FORCE
	cd kernel; \
	cargo xbuild --release --target target.json
	cp kernel/target/target/release/kernel $(BUILD_DIR)/kernel.o

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
	$(MAKE) -C $(SRC_DIR)/bootloader/primary BUILD_DIR=$(abspath $(BUILD_DIR)) clean
	$(MAKE) -C $(SRC_DIR)/bootloader/secondary BUILD_DIR=$(abspath $(BUILD_DIR)) clean
	rm -rf $(BUILD_DIR)/*

run: $(BUILD_DIR)/boot_floppy.img
	qemu-system-x86_64 -fda $(BUILD_DIR)/boot_floppy.img

debug: $(BUILD_DIR)/boot_floppy.img
	bochs -f bochs.cfg -q