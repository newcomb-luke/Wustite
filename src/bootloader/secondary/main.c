#include <stdint.h>
#include "bio.h"
#include "fat.h"
#include "bdisk.h"
#include "memory.h"
#include "elf.h"
#include "long_mode.h"
#include "paging.h"
#include "smap.h"

void __attribute__((cdecl)) _start(uint32_t bootDrive) {
	setVideoMode(Text80x25_Color);

    DISK disk;
    FAT12_Index index;
    FAT12_Index* indexPtr = (FAT12_Index*) &index;

    if (DISK_Initialize(&disk, bootDrive) != 0) {
        puts("Failed to initialize disk");
        for (;;) {}
    }

    puts("Disk initialized");

    // Our FAT file system starts where we were loaded into memory
    if (FAT_DRIVER_INIT(&disk, indexPtr, (uint8_t*)(FAT_CURRENT_DIRECTORY_BUFFER_START), (uint8_t*)(FAT_CURRENT_FAT_SECTION_BUFFER_START)) != 0) {
        puts("Error initializing FAT driver");
        for (;;) {}
    }

    puts("FAT12 driver initialized");

    // Print the volume label as a test
    printf("Volume label: ");
    char volumeLabel[11];
    readVolumeLabel((char*) &volumeLabel);

    for (int i = 0; i < 11; i++) {
        putc(volumeLabel[i]);
    }

    putc('\n');

    FAT12_FILE file;
    uint8_t* fileBuffer = (uint8_t*)(FAT_FILE_BUFFER_START);
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

    if (!is_cpuid_available() || !is_extended_cpuid_available()) {
        puts("Kernel requires x86_64.");
        for (;;) {}
    }

    puts("CPUID is supported");

    puts("Loading kernel");

    // Set up the page table, even though we don't use it quite yet
    identityMap(NUM_MEGABYTES_TO_MAP);

    // Store the boot drive in the standard predefined memory location
    uint32_t* bootDrivePtr = (uint32_t*)(BOOT_DRIVE_MEM_LOC);
    *bootDrivePtr = bootDrive;

    uint32_t* smapEntryCount = (uint32_t*)(SMAP_NUM_ENTRIES_LOC);
    SMAPEntry* smapEntriesStart = (SMAPEntry*)(SMAP_FIRST_ENTRY_LOC);

    detectMemory(smapEntryCount, smapEntriesStart);

    loadAndExecuteELF(fileBuffer);

    for (;;) {}
}

