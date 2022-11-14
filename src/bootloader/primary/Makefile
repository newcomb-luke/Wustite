BUILD_DIR?=build/
ASM?=nasm

.PHONY: all primary clean

all: primary

primary: $(BUILD_DIR)/primary.bin

$(BUILD_DIR)/primary.bin: primary.asm
	$(ASM) -fbin primary.asm -o $(BUILD_DIR)/primary.bin

clean:
	rm -f $(BUILD_DIR)/primary.bin
