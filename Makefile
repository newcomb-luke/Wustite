.PHONY: all clean bootloader kernel

all: hard_disk

BUILD_DIR=$(abspath build)
TARGET_DIR=$(abspath target)

UEFI_PARTITION=$(BUILD_DIR)/uefi-partition.img
UEFI_PARTITION_SIZE=91669

HARD_DISK_IMG=$(BUILD_DIR)/hard_disk.img
HARD_DISK_SIZE=93750

INITRAMFS=$(BUILD_DIR)/ramfs.img
INITRAMFS_SIZE=20480

GPT_OFFSET=2048

BOOTLOADER_BUILD_STD=core
BOOTLOADER_TARGET_NAME=x86_64-unknown-uefi
BOOTLOADER_TARGET=$(BOOTLOADER_TARGET_NAME)
BOOTLOADER_OUTPUT=$(TARGET_DIR)/$(BOOTLOADER_TARGET)/release/bootloader-uefi.efi

KERNEL_RUST_FLAGS=-C code-model=kernel -C relocation-model=pic
KERNEL_BUILD_STD=core,alloc
KERNEL_TARGET=x86_64-none-eabi
KERNEL_TARGET_NAME=$(KERNEL_TARGET).json
KERNEL_OUTPUT=$(TARGET_DIR)/$(KERNEL_TARGET)/release/kernel

MODULE_RUST_FLAGS=-C code-model=kernel -C relocation-model=pic
MODULE_BUILD_STD=core
MODULE_TARGET=x86_64-none-eabi
MODULE_TARGET_NAME=$(MODULE_TARGET).json
MODULE_OUTPUT_DIR=target/$(MODULE_TARGET)/release

#
# Bootloader
#

bootloader: $(BUILD_DIR)/BOOTX64.EFI

$(BUILD_DIR)/BOOTX64.EFI: efi_partition FORCE
	cargo build --release -Zbuild-std=$(BOOTLOADER_BUILD_STD) --target=$(BOOTLOADER_TARGET_NAME) --package=bootloader-uefi
	cp $(BOOTLOADER_OUTPUT) $(BUILD_DIR)/BOOTX64.EFI
	mcopy -i $(UEFI_PARTITION) $(BUILD_DIR)/BOOTX64.EFI "::EFI/BOOT/BOOTX64.EFI" -o

#
# EFI Partition
#

efi_partition: $(UEFI_PARTITION)

$(UEFI_PARTITION):
	mkdir -p $(BUILD_DIR)
	dd if=/dev/zero of=$(BUILD_DIR)/uefi-partition.img bs=512 count=$(UEFI_PARTITION_SIZE)
	mformat -i $(UEFI_PARTITION) -h 32 -t 32 -n 64 -c 1
	mmd -i $(UEFI_PARTITION) "::EFI"
	mmd -i $(UEFI_PARTITION) "::EFI/BOOT"

# 
# Hard disk image
#

hard_disk: raw_disk kernel modules
	dd if=$(UEFI_PARTITION) of=$(HARD_DISK_IMG) bs=512 count=$(UEFI_PARTITION_SIZE) seek=$(GPT_OFFSET) conv=notrunc

raw_disk: $(HARD_DISK_IMG)

$(HARD_DISK_IMG):
	mkdir -p $(BUILD_DIR)
	dd if=/dev/zero of=$(HARD_DISK_IMG) bs=512 count=$(HARD_DISK_SIZE)
	parted $(HARD_DISK_IMG) -s -a minimal mklabel gpt
	parted $(HARD_DISK_IMG) -s -a minimal mkpart EFI FAT16 $(GPT_OFFSET)s 93716s
	parted $(HARD_DISK_IMG) -s -a minimal toggle 1 boot

# 
# Kernel
#

kernel: $(BUILD_DIR)/kernel.o

$(BUILD_DIR)/kernel.o: bootloader FORCE
	mkdir -p $(BUILD_DIR)
#	RUSTFLAGS="$(KERNEL_RUST_FLAGS)" cargo build --release -Z build-std=$(KERNEL_BUILD_STD) --target=$(KERNEL_TARGET_NAME) --package=kernel
	cp $(KERNEL_OUTPUT) $(BUILD_DIR)/kernel.o
	mcopy -i $(UEFI_PARTITION) $(BUILD_DIR)/kernel.o "::kernel.o" -o

# 
# Initramfs
#

initramfs: $(INITRAMFS)

$(INITRAMFS): efi_partition
	dd if=/dev/zero of=$(INITRAMFS) bs=512 count=$(INITRAMFS_SIZE)
	mkfs.fat -F 16 -n "INITRAMF" $(INITRAMFS)
	mcopy -i $(UEFI_PARTITION) $(INITRAMFS) "::INITRAMF.IMG" -o

#
# Kernel modules
#

modules: ide_driver

ide_driver: $(BUILD_DIR)/libide_driver.so

$(BUILD_DIR)/libide_driver.so: initramfs
	cd modules/ide_driver && \
	RUSTFLAGS="$(MODULE_RUST_FLAGS)" cargo build --release -Z build-std=$(MODULE_BUILD_STD) --target=../../$(MODULE_TARGET_NAME) && \
	cp $(MODULE_OUTPUT_DIR)/libide_driver.so $(BUILD_DIR)/libide_driver.so
	mcopy -i $(INITRAMFS) $(BUILD_DIR)/libide_driver.so "::libide.so" -o

#
# QEMU Firmware
#

firmware: $(BUILD_DIR)/OVMF_VARS.fd

$(BUILD_DIR)/OVMF_VARS.fd:
	cp /usr/share/edk2-ovmf/x64/OVMF_VARS.fd $(BUILD_DIR)

run: hard_disk firmware
	qemu-system-x86_64 --enable-kvm -cpu host,pdpe1gb=on -m 2G \
		-device vmware-svga \
		-drive if=pflash,format=raw,readonly=on,file=/usr/share/edk2-ovmf/x64/OVMF_CODE.fd \
		-drive if=pflash,format=raw,file=$(BUILD_DIR)/OVMF_VARS.fd \
		-drive if=ide,format=raw,file=$(HARD_DISK_IMG)

FORCE: ;

clean:
	cargo clean
	cd modules/ide_driver && cargo clean
	rm -drf build
