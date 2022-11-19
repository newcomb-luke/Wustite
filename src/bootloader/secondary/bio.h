#pragma once

#include <stdint.h>

enum VideoMode {
	Text40x25_Gray = 0x00,
	Text40x25_Color = 0x01,
	Text80x25_Gray = 0x02,
	Text80x25_Color = 0x03,
	Graphics320x200_4Color = 0x04,
	Graphics320x200_4Gray = 0x05,
	Graphics640x200_2Color = 0x06,
	Text80x25_BW = 0x07,
	Graphics160x200_pcjr_16Color = 0x08,
	Graphics320x200_pcjr_16Color = 0x09,
	Graphics640x200_pcjr_4Color = 0x0a,
	Graphics320x200_16Color = 0x0d,
	Graphics640x200_16Color = 0x0e,
	Graphics620x350_BW = 0x0f,
	Graphics620x350_16Color = 0x10
};

void putc(char c);
void puts(const char* s);
void printf(const char* s);
void setVideoMode(enum VideoMode m);
void phexuint8(uint8_t value);
void phexuint16(uint16_t value);
void phexuint32(uint32_t value);
void phexuint64(uint64_t value);
void hexdump(uint8_t* addr);
