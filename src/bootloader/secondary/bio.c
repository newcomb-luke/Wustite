#include "bio.h"
#include "_bios.h"

void putc(char c) {
    if (c == '\n') {
        _BIOS_Video_WriteCharTeletype('\r');
        _BIOS_Video_WriteCharTeletype('\n');
    } else {
        _BIOS_Video_WriteCharTeletype(c);
    }
}

void puts(const char* s) {
	while (*s != 0) {
		putc(*(s++));
	}
    putc('\n');
}

void printf(const char* s) {
    while (*s != 0) {
        putc(*(s++));
    }
}

void setVideoMode(enum VideoMode m) {
	_BIOS_Video_SetVideoMode(m);
}

const char* HEX_MAP = "0123456789abcdef";

void phexuint8(uint8_t value) {
    uint8_t shift = 8 - 4;

    for (int i = 0; i < 2; i++) {
        uint8_t nibble = (value >> shift) & 0xF;
        putc(HEX_MAP[nibble]);
        shift -= 4;
    }
}

void phexuint16(uint16_t value) {
    uint8_t shift = 16 - 4;

    for (int i = 0; i < 4; i++) {
        uint8_t nibble = (value >> shift) & 0xF;
        putc(HEX_MAP[nibble]);
        shift -= 4;
    }
}

void phexuint32(uint32_t value) {
    uint8_t shift = 32 - 4;

    for (int i = 0; i < 8; i++) {
        uint8_t nibble = (value >> shift) & 0xF;
        putc(HEX_MAP[nibble]);
        shift -= 4;
    }
}

void phexuint64(uint64_t value) {
    uint8_t shift = 64 - 4;

    for (int i = 0; i < 16; i++) {
        uint8_t nibble = (value >> shift) & 0xF;
        putc(HEX_MAP[nibble]);
        shift -= 4;
    }
}

void hexdump(uint8_t* addr) {
    for (int i = 0; i < 20; i++) {
        for (int j = 0; j < 26; j++) {
            phexuint8(*addr);
            putc(' ');
            addr++;
        }
        putc('\n');
    }
}
