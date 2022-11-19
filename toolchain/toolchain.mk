toolchain: toolchain_binutils toolchain_gcc

BINUTILS_VERSION=2.39
GCC_VERSION=12.2.0
BINUTILS_URL=https://ftp.gnu.org/gnu/binutils/binutils-$(BINUTILS_VERSION).tar.gz
GCC_URL=https://ftp.gnu.org/gnu/gcc/gcc-12.2.0/gcc-$(GCC_VERSION).tar.gz
BUILD_THREADS=8
TOOLCHAIN_BUILD=toolchain/build
BINUTILS_BUILD=$(TOOLCHAIN_BUILD)/binutils-$(BINUTILS_VERSION)-build
GCC_BUILD=$(TOOLCHAIN_BUILD)/gcc-$(BINUTILS_VERSION)-build

export TARGET_CC = $(TOOLCHAIN_PREFIX)/bin/$(TARGET)-gcc
export TARGET_LD = $(TOOLCHAIN_PREFIX)/bin/$(TARGET)-gcc

toolchain_binutils:
	mkdir -p $(TOOLCHAIN_BUILD)
	cd $(TOOLCHAIN_BUILD); \
	wget $(BINUTILS_URL); \
	tar -xf binutils-$(BINUTILS_VERSION).tar.gz
	mkdir -p $(BINUTILS_BUILD)
	cd $(BINUTILS_BUILD); \
	../binutils-$(BINUTILS_VERSION)/configure \
		--prefix="$(TOOLCHAIN_PREFIX)" \
		--target=$(TARGET)             \
		--with-sysroot                 \
		--disable-nls                  \
		--disable-werror
	$(MAKE) -j$(BUILD_THREADS) -C $(BINUTILS_BUILD)
	$(MAKE) -C $(BINUTILS_BUILD) install

toolchain_gcc: toolchain_binutils
	mkdir -p $(TOOLCHAIN_BUILD)
	cd $(TOOLCHAIN_BUILD); \
	wget $(GCC_URL); \
	tar -xf gcc-$(GCC_VERSION).tar.gz
	mkdir -p $(GCC_BUILD)
	cd $(GCC_BUILD); \
	../gcc-$(GCC_VERSION)/configure \
		--prefix="$(TOOLCHAIN_PREFIX)" \
		--target=$(TARGET)             \
		--disable-nls                  \
		--enable-languages=c,c++       \
		--disable-hosted-libstdcxx     \
		--without-headers
	$(MAKE) -j$(BUILD_THREADS) -C $(GCC_BUILD) all-gcc all-target-libgcc
	$(MAKE) -C $(GCC_BUILD) install-gcc install-target-libgcc
