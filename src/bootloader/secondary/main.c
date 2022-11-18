#include "stdint.h"
#include "bio.h"
#include "fat.h"
#include "bdisk.h"
#include "string.h"
#include "memory.h"
#include "elf.h"
#include "_x86.h"
#include "_bios.h"

void _cdecl cstart_(uint16_t bootDrive) {
	setVideoMode(Text80x25_Color);

    DISK disk;
    FAT12_Index index;
    FAT12_Index far* indexPtr = (FAT12_Index far*) &index;

    if (DISK_Initialize(&disk, bootDrive) != 0) {
        puts("Failed to initialize disk");
        for (;;) {}
    }

    puts("Disk initialized");

    // Our FAT file system starts where we were loaded into memory
    if (FAT_DRIVER_INIT(&disk, indexPtr, (uint8_t far*)(FAT_CURRENT_DIRECTORY_BUFFER_START), (uint8_t far*)(FAT_CURRENT_FAT_SECTION_BUFFER_START)) != 0) {
        puts("Error initializing FAT driver");
        for (;;) {}
    }

    puts("FAT12 driver initialized");

    // Print the volume label as a test
    printf("Volume label: ");
    char volumeLabel[11];
    readVolumeLabel((char far*) &volumeLabel);

    for (int i = 0; i < 11; i++) {
        putc(volumeLabel[i]);
    }

    putc('\n');

    FAT12_FILE file;
    uint8_t far* fileBuffer = (uint8_t far*)(FAT_FILE_BUFFER_START);
    uint32_t bytesRead = 0;
    const char* fileName = "kernel.o";

    if (openFile(&disk, indexPtr, &file, fileName) != 0) {
        printf("Failed to open ");
        puts(fileName);
        for (;;) {}
    }

    if (readFile(&disk, indexPtr, &file, fileBuffer, file.size, &bytesRead) != 0) {
        printf("Failed to read ");
        puts(fileName);
        for (;;) {}
    }

    printf("Bytes read: 0x");
    phexuint32(bytesRead);
    putc('\n');

    if (readELF(fileBuffer) != 0) {
        printf("Failed to read ");
        puts(fileName);
        for (;;) {}
    }

    puts("ELF file read.");

    if (_enable_a20() != 0) {
        puts("Failed to enable A20 line");
        for (;;) {}
    }

    puts("A20 line enabled");

	for (;;) {}
}

