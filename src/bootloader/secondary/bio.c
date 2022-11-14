#include "bio.h"
#include "_bios.h"

void putc(char c) {
	_BIOS_Video_WriteCharTeletype(c, 0);
}

void puts(const char* s) {
	while (*s != 0) {
		putc(*(s++));
	}
}

void setVideoMode(enum VideoMode m) {
	_BIOS_Video_SetVideoMode(m);
}
