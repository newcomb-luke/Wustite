BUILD_DIR?=build/
ASM?=nasm
ASMFLAGS=-f obj
CC_16=/opt/watcom/binl/wcc
CFLAGS_16=-4 -d3 -s -wx -ms -zl -zq
LD_16=/opt/watcom/binl/wlink

SOURCES_C=$(wildcard *.c)
SOURCES_ASM=$(wildcard *.asm)
OBJECTS_C=$(pathsubst %.c, $(BUILD_DIR)/secondary/c/%.obj, $(SOURCES_C))
OBJECTS_ASM=$(pathsubst %.asm, $(BUILD_DIR)/secondary/asm/%.obj, $(SOURCES_ASM))

.PHONY: all secondary clean always

all: secondary

secondary: $(BUILD_DIR)/secondary.bin

$(BUILD_DIR)/secondary.bin: $(OBJECTS_ASM) $(OBJECTS_C)
	$(LD_16) NAME $(BUILD_DIR)/secondary.bin FILE \{ $(OBJECTS_ASM) $(OBJECTS_C) \} OPTION MAP=$(BUILD_DIR)/secondary.map @linker.lnk

$(BUILD_DIR)/secondary/c/%.obj: %.c always
	$(CC_16) $(CFLAGS_16) -fo=$@ $<

$(BUILD_DIR)/secondary/asm/%.obj: %.asm always
	$(ASM) $(ASMFLAGS) -o $@ $<

always:
	mkdir -p $(BUILD_DIR)/secondary/c
	mkdir -p $(BUILD_DIR)/secondary/asm

clean:
	rm -f $(BUILD_DIR)/secondary.bin
	rm -drf $(BUILD_DIR)/secondary
