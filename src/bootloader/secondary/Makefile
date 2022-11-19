TARGET_ASMFLAGS += -f elf
TARGET_CFLAGS += -ffreestanding -nostdlib
TARGET_LIBS += -lgcc
TARGET_LINKFLAGS += -T linker.ld -nostdlib

SOURCES_C=$(wildcard *.c)
SOURCES_ASM=$(wildcard *.asm)
OBJECTS_C=$(patsubst %.c, $(BUILD_DIR)/secondary/c/%.o, $(SOURCES_C))
OBJECTS_ASM=$(patsubst %.asm, $(BUILD_DIR)/secondary/asm/%.o, $(SOURCES_ASM))

.PHONY: all secondary clean always

all: secondary

secondary: $(BUILD_DIR)/secondary.bin

$(BUILD_DIR)/secondary.bin: $(OBJECTS_ASM) $(OBJECTS_C)
	$(TARGET_LD) $(TARGET_LINKFLAGS) -Wl,-Map=$(BUILD_DIR)/secondary.map -o $@ $^ $(TARGET_LIBS)

$(BUILD_DIR)/secondary/c/%.o: %.c always
	$(TARGET_CC) $(TARGET_CFLAGS) -c -o $@ $<

$(BUILD_DIR)/secondary/asm/%.o: %.asm always
	$(TARGET_ASM) $(TARGET_ASMFLAGS) -o $@ $<

always:
	mkdir -p $(BUILD_DIR)/secondary/c
	mkdir -p $(BUILD_DIR)/secondary/asm

clean:
	rm -f $(BUILD_DIR)/secondary.bin
	rm -drf $(BUILD_DIR)/secondary

