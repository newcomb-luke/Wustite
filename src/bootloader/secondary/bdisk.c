#include "bdisk.h"
#include "_bios.h"
#include "bio.h"

#define SECTOR_SIZE 512

uint16_t DISK_Initialize(DISK* disk, uint8_t driveNumber) {
    disk->driveNumber = driveNumber;

    if (_BIOS_Drive_GetParams(driveNumber,
                          &disk->driveType,
                          &disk->maxHead,
                          &disk->maxCylinder,
                          &disk->maxSector) != 0) {
        return 1;
    }

    /*
    printf("Cylinders: ");
    phexuint16(disk->maxCylinder + 1);
    putc('\n');

    printf("Heads: ");
    phexuint8(disk->maxHead + 1);
    putc('\n');

    printf("Sectors: ");
    phexuint8(disk->maxSector);
    putc('\n');
    */

    return 0;
}

uint16_t DISK_Reset(DISK* disk) {
    return _BIOS_Drive_Reset(disk->driveNumber);
}

void DISK_LBA_to_CHS(DISK* disk,
                     uint32_t lba,
                     uint8_t* headOut,
                     uint16_t* cylinderOut,
                     uint8_t* sectorOut) {
    // sector = (LBA % sectors per cylinder + 1)
    *sectorOut = (lba % disk->maxSector) + 1;
    // cylinder = (LBA / sectors per cylinder) / heads on disk
    *cylinderOut = lba / (disk->maxSector * (disk->maxHead + 1));
    // head = (LBA / sectors per cylinder) % heads on disk
    *headOut = (lba / (disk->maxSector)) % (disk->maxHead + 1);
}

uint16_t DISK_Read(DISK* disk, uint32_t lba, uint8_t sectorsToRead, uint8_t far* destination) {
    uint8_t head;
    uint16_t cylinder;
    uint8_t sector;

    /*
    printf("LBA: ");
    phexuint32(lba);
    putc('\r');
    putc('\n');
    */

    DISK_LBA_to_CHS(disk, lba, &head, &cylinder, &sector);

    /*
    printf("Cylinder: ");
    phexuint16(cylinder);
    putc('\n');

    printf("Head: ");
    phexuint8(head);
    putc('\n');

    printf("Sector: ");
    phexuint8(sector);
    putc('\n');
    */

    for (uint16_t i = 0; i < 3; i++) {
        if (_BIOS_Drive_ReadSectors(disk->driveNumber,
                                   head,
                                   cylinder,
                                   sector,
                                   sectorsToRead,
                                   destination) == 0) {
            return 0;
        }

        if (_BIOS_Drive_Reset(disk->driveNumber) != 0) {
            puts("Drive failed to reset");
            return 1;
        }
    }

    puts("Drive read failed");
    return 1;
}
