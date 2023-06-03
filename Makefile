.PHONY: all disk_image efi_partition bootloader clean always kernel initramfs modules firmware

all: efi_partition bootloader disk_image kernel initramfs modules firmware

BUILD_DIR=$(abspath build)

#
# Bootloader
#

bootloader: $(BUILD_DIR)/BOOTX64.EFI

$(BUILD_DIR)/BOOTX64.EFI: always FORCE
	cargo build --release -Zbuild-std=core,compiler_builtins --target x86_64-unknown-uefi --package=bootloader-uefi
	cp target/x86_64-unknown-uefi/release/bootloader-uefi.efi $(BUILD_DIR)/BOOTX64.EFI

#
# EFI partition
#
efi_partition: $(BUILD_DIR)/uefi-partition.img

$(BUILD_DIR)/uefi-partition.img: bootloader
	dd if=/dev/zero of=$(BUILD_DIR)/uefi-partition.img bs=512 count=91669
	mformat -i $(BUILD_DIR)/uefi-partition.img -h 32 -t 32 -n 64 -c 1
	mmd -i $(BUILD_DIR)/uefi-partition.img ::EFI
	mmd -i $(BUILD_DIR)/uefi-partition.img ::EFI/BOOT
	mcopy -i $(BUILD_DIR)/uefi-partition.img $(BUILD_DIR)/BOOTX64.EFI ::EFI/BOOT

#
# Disk image
#

disk_image: $(BUILD_DIR)/boot_disk.img

$(BUILD_DIR)/boot_disk.img: efi_partition kernel initramfs modules
	dd if=/dev/zero of=$(BUILD_DIR)/boot_disk.img bs=512 count=93750
	parted $(BUILD_DIR)/boot_disk.img -s -a minimal mklabel gpt
	parted $(BUILD_DIR)/boot_disk.img -s -a minimal mkpart EFI FAT16 2048s 93716s
	parted $(BUILD_DIR)/boot_disk.img -s -a minimal toggle 1 boot
	dd if=$(BUILD_DIR)/uefi-partition.img of=$(BUILD_DIR)/boot_disk.img bs=512 count=91669 seek=2048 conv=notrunc
# 	mcopy -i $(BUILD_DIR)/boot_disk.img $(BUILD_DIR)/kernel.o "::kernel.o"
# 	mcopy -i $(BUILD_DIR)/boot_disk.img $(BUILD_DIR)/ramfs.bin "::ramfs.bin"

#
# Firmware image
# 
firmware: $(BUILD_DIR)/OVMF_VARS.fd

$(BUILD_DIR)/OVMF_VARS.fd:
	cp /usr/share/edk2-ovmf/x64/OVMF_VARS.fd $(BUILD_DIR)

# 
# Kernel
#
kernel: $(BUILD_DIR)/kernel.o

$(BUILD_DIR)/kernel.o: always FORCE
	RUSTFLAGS="-C code-model=kernel -C relocation-model=static" cargo build --release -Z build-std=core,alloc --target=x86_64-none-eabi.json --package=kernel
	cp target/x86_64-none-eabi/release/kernel $(BUILD_DIR)/kernel.o

#
# Initramfs
#
initramfs: modules
	dd if=/dev/zero of=$(BUILD_DIR)/ramfs.bin bs=512 count=128
	mkfs.fat -F 12 -n "INITRAM " $(BUILD_DIR)/ramfs.bin
	mcopy -i $(BUILD_DIR)/ramfs.bin $(BUILD_DIR)/libide_driver.so "::libide.so"

#
# Kernel modules
#
modules: $(BUILD_DIR)/ide_driver.so

$(BUILD_DIR)/ide_driver.so: 
	cd modules/ide_driver && \
	RUSTFLAGS="-C code-model=kernel -C relocation-model=pic" cargo build --release -Z build-std=core --target=../../x86_64-none-eabi.json && \
	cp target/x86_64-none-eabi/release/libide_driver.so $(BUILD_DIR)/libide_driver.so

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

run: $(BUILD_DIR)/boot_disk.img firmware
	qemu-system-x86_64 --enable-kvm -cpu host,pdpe1gb=on \
		-device vmware-svga -m 2G \
		-drive if=pflash,format=raw,readonly=on,file=/usr/share/edk2-ovmf/x64/OVMF_CODE.fd \
		-drive if=pflash,format=raw,file=$(BUILD_DIR)/OVMF_VARS.fd \
		-drive if=ide,format=raw,file=$(BUILD_DIR)/boot_disk.img

debug: $(BUILD_DIR)/boot_disk.img
	bochs -f bochs.cfg -q
