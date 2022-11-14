#include "stdint.h"
#include "bio.h"

void _cdecl cstart_(uint16_t bootDrive) {

	setVideoMode(Text80x25_Color);

	puts("Hello from the other side!");

	for (;;) {
		//
	}
}
