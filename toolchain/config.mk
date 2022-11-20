export TARGET=i686-elf
export TARGET_SYS_INCLUDE = $(TOOLCHAIN_PREFIX)/lib/gcc/$(TARGET)/$(GCC_VERSION)/include
TOOLCHAIN_PREFIX=$(abspath toolchain/$(TARGET))

export TARGET_CFLAGS= -std=c99 -g
export TARGET_ASM=nasm
export TARGET_ASMFLAGS=
export TARGET_LIBS=
export TARGET_LINKFLAGS=

SRC_DIR=src
BUILD_DIR=$(abspath build)
KERNEL_BASE_DIR=$(abspath kernel)