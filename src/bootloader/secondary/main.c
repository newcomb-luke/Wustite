#include "stdint.h"
#include "bio.h"
#include "fat.h"
#include "string.h"

void _cdecl cstart_(uint16_t bootDrive) {
	setVideoMode(Text80x25_Color);

    // Our FAT file system starts where we were loaded into memory
    initFAT((char far*) 0x00007c00);

    // Print the volume label as a test
    puts("Volume label: ");
    char volumeLabel[11];
    readVolumeLabel((char far*) &volumeLabel);

    for (int i = 0; i < 11; i++) {
        putc(volumeLabel[i]);
    }

    putc('\r');
    putc('\n');

	for (;;) {}
}

