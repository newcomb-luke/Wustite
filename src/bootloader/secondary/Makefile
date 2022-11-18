BUILD_DIR?=build/
ASM?=nasm
ASMFLAGS=-f obj
CC_16=/opt/watcom/binl/wcc
CC_32=/opt/watcom/binl/wcc386
CFLAGS_16=-4 -d3 -s -wx -ms -zl -zq -za99
LD_16=/opt/watcom/binl/wlink

SOURCES_C=$(wildcard *.c)
SOURCES_ASM=$(wildcard *.asm)
OBJECTS_C=$(patsubst %.c, $(BUILD_DIR)/secondary/c/%.obj, $(SOURCES_C))
OBJECTS_ASM=$(patsubst %.asm, $(BUILD_DIR)/secondary/asm/%.obj, $(SOURCES_ASM))

.PHONY: all secondary clean always

all: secondary

secondary: $(BUILD_DIR)/secondary.bin

$(BUILD_DIR)/secondary.bin: $(OBJECTS_ASM) $(OBJECTS_C) $(BUILD_DIR)/secondary/c32/elf2.obj
	$(LD_16) NAME $(BUILD_DIR)/secondary.bin FILE \{ $(OBJECTS_ASM) $(OBJECTS_C) $(BUILD_DIR)/secondary/c32/elf2.obj \} OPTION MAP=$(BUILD_DIR)/stage2.map @linker.lnk

$(BUILD_DIR)/secondary/c/%.obj: %.c always
	$(CC_16) $(CFLAGS_16) -fo=$@ $<

$(BUILD_DIR)/secondary/asm/%.obj: %.asm always
	$(ASM) $(ASMFLAGS) -o $@ $<

$(BUILD_DIR)/secondary/c32/elf2.obj: bits32/elf2.c always
	$(CC_32) $(CFLAGS_16) -fo=$(BUILD_DIR)/secondary/c32/elf2.obj bits32/elf2.c

always:
	mkdir -p $(BUILD_DIR)/secondary/c
	mkdir -p $(BUILD_DIR)/secondary/c32
	mkdir -p $(BUILD_DIR)/secondary/asm

clean:
	rm -f $(BUILD_DIR)/secondary.bin
	rm -drf $(BUILD_DIR)/secondary
