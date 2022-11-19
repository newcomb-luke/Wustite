#pragma once

#include <stdint.h>

typedef struct {
    uint8_t driveNumber;
    uint8_t driveType;
    uint8_t maxHead;
    uint16_t maxCylinder;
    uint8_t maxSector;
} DISK;

uint16_t DISK_Initialize(DISK* disk, uint8_t driveNumber);

uint16_t DISK_Reset(DISK* disk);

uint16_t DISK_Read(DISK* disk, uint32_t lba, uint8_t sectorsToRead, uint8_t* destination);

void DISK_LBA_to_CHS(DISK* disk, uint32_t lba, uint8_t* headOut, uint16_t* cylinderOut, uint8_t* sectorOut);
