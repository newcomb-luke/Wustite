#include "fat.h"
#include "stdint.h"
#include "string.h"
#include "bio.h"
#include "ctype.h"
#include "bdisk.h"

#define SECTOR_SIZE 512
#define FILE_NAME_LEN 11

#pragma pack(push, 1)
typedef enum {
    READ_ONLY = 0x01,
    HIDDEN = 0x02,
    SYSTEM = 0x04,
    VOLUME_ID = 0x08,
    DIRECTORY = 0x10,
    ARCHIVE = 0x20,
    LFN = READ_ONLY | HIDDEN | SYSTEM | VOLUME_ID
} DirectoryAttribute;
#pragma pack(pop)

#pragma pack(push, 1)
typedef enum {
    FREE_CLUSTER = 0x000,
    BAD_CLUSTER = 0xFF7,
    LAST_CLUSTER = 0xFF8,
} FAT12Cluster;
#pragma pack(pop)

// Just in case something horrible happens to the boot sector
static uint8_t g_FATBootRecordBuffer[SECTOR_SIZE];
static FAT12_BootRecord* g_FATBootRecord = (FAT12_BootRecord*) &g_FATBootRecordBuffer;

uint16_t _LoadRootDirectory(DISK* disk, FAT12_Index far* index);

uint16_t FAT_DRIVER_INIT(DISK* disk, FAT12_Index far* index, uint8_t far* currentDirectoryBuffer, uint8_t far* currentFATSectionBuffer) {
    index->currentDirectoryBuffer = (FAT12_DirEntry far*) currentDirectoryBuffer;
    index->currentFATSectionBuffer = currentFATSectionBuffer;

    if (DISK_Read(disk, 0, 1, (uint8_t far*) (g_FATBootRecord)) != 0) {
        puts("Could not read boot sector");
        return 1;
    }

    index->FATStartSector = g_FATBootRecord->bdb_reserved_sectors;

    index->rootDirStartSector = index->FATStartSector + g_FATBootRecord->bdb_fat_count * g_FATBootRecord->bdb_sectors_per_fat;

    // This calculation rounds up to the nearest whole sector, which is how the data is stored if it doesn't fit neatly
    index->dataRegionStartSector = index->rootDirStartSector + ((g_FATBootRecord->bdb_dir_entries_count * 32) + (g_FATBootRecord->bdb_bytes_per_sector - 1)) / SECTOR_SIZE;

    // Our current directory buffer is 2 sectors long, even if the directory entry is larger/smaller
    if (_LoadRootDirectory(disk, index) != 0) {
        puts("Loading root directory failed");
        return 2;
    }

    if (DISK_Read(disk, index->FATStartSector, g_FATBootRecord->bdb_sectors_per_fat, (uint8_t far*) index->currentFATSectionBuffer) != 0) {
        puts("Loading FAT failed");
        return 3;
    }

    return 0;
}

uint32_t _ClusterToLBA(FAT12_Index far* index, uint16_t cluster) {
    return index->dataRegionStartSector + (cluster - 2) * g_FATBootRecord->bdb_sectors_per_cluster;
}

void readOEM(char far* buffer) {
    memcpy((void far*) &(g_FATBootRecord->bdb_oem_id), (void far*) buffer, 8);
}

void readVolumeLabel(char far* buffer) {
    memcpy((void far*) &g_FATBootRecord->ebr_volume_label, (void far*) buffer, 11);
}

uint16_t _LoadRootDirectory(DISK* disk, FAT12_Index far* index) {
    if (DISK_Read(disk, index->rootDirStartSector, 2, (uint8_t far*) index->currentDirectoryBuffer) != 0) {
        return 1;
    }

    index->currentDirectoryBufferStartSector = index->rootDirStartSector;

    return 0;
}

uint16_t _CStrTo8Point3(const uint8_t* fileName, uint8_t* nameBuffer) {
    uint16_t nameLen = strlen(fileName);

    // size of 12 counts the . that isn't present in the 8.3 filename
    if (nameLen > 12 || nameLen < 1) {
        puts("File name too long or empty");
        return 1;
    }

    const uint8_t* sep = strnchr(fileName, '.', FILE_NAME_LEN + 1);

    memset((void far*) nameBuffer, ' ', FILE_NAME_LEN);

    if (sep == NULL) {
        if (nameLen > 8) {
            return 2;
        }

        // Just copy the plain name over
        memcpy((void far*) fileName, (void far*) nameBuffer, nameLen);
    } else {
        uint8_t namePartLen = (uint8_t)(sep - (uint8_t*) fileName);
        // -1 for the dot before the file extension
        uint8_t extPartLen = nameLen - namePartLen - 1;

        if (namePartLen > 8 || extPartLen > 3) {
            puts("Invalid file name length");
            return 3;
        }

        const uint8_t* extPtr = &fileName[namePartLen + 1];

        memcpy((void far*) fileName, (void far*) nameBuffer, namePartLen);

        memcpy((void far*) extPtr, (void far*) &nameBuffer[8], extPartLen);
    }

    for (uint8_t i = 0; i < FILE_NAME_LEN; i++) {
        nameBuffer[i] = toupper(nameBuffer[i]);
    }

    return 0;
}

FAT12_DirEntry far* _FindEntryInRootDirectory(DISK* disk, FAT12_Index far* index, uint8_t* name) {
    for(;;) {
        // Our entry buffer can store up to 32 entries.
        // If we read all 32, and there is no entry marking the end,
        // then we have to load the next section of it into the buffer.
        for (int i = 0; i < 32; i++) {
            FAT12_DirEntry far* entry = (FAT12_DirEntry far*) &index->currentDirectoryBuffer[i];

            // This marks the end of the directory table
            if (entry->entryName[0] == '\0') {
                printf("Could not find ");

                for (int i = 0; i < FILE_NAME_LEN; i++) {
                    putc(name[i]);
                }

                puts(" in directory table");
                // At this point, we haven't found it
                return NULL;
            }

            if ((entry->attributes & DIRECTORY) != 0) {
                printf("Has directory: ");
            } else if ((entry->attributes & VOLUME_ID) != 0) {
                printf("Has volume id: ");
            } else {
                printf("Has file: ");
            }

            for (int i = 0; i < FILE_NAME_LEN; i++) {
                putc(entry->entryName[i]);
            }

            putc('\n');

            if (memcmp((void far*) &entry->entryName, (void far*) name, FILE_NAME_LEN) == 0) {
                return entry;
            }
        }

        // Read the next section of it
        index->currentDirectoryBufferStartSector += 2;

        if (DISK_Read(disk, index->currentDirectoryBufferStartSector, 2, (uint8_t far*) index->currentDirectoryBuffer) != 0) {
            puts("Failed to read next section of directory");
            return NULL;
        }
    }

    return NULL;
}

uint16_t openFile(DISK* disk, FAT12_Index far* index, FAT12_FILE* fileOut, const char* fileName) {
    uint8_t FAT12FileNameBuffer[FILE_NAME_LEN];

    // We will always begin by re-loading the root directory
    if (_LoadRootDirectory(disk, index) != 0) {
        return 1;
    }

    _CStrTo8Point3(fileName, &FAT12FileNameBuffer);

    FAT12_DirEntry far* entry = _FindEntryInRootDirectory(disk, index, &FAT12FileNameBuffer);

    if (entry == NULL) {
        return 2;
    }

    if ((entry->attributes & DIRECTORY) != 0) {
        puts("Subdirectories are not supported");
        return 3;
    }

    fileOut->startCluster = entry->firstClusterLow;
    fileOut->currentCluster = entry->firstClusterLow;
    fileOut->size = entry->fileSize;

    return 0;
}

uint16_t _ReadCluster(DISK* disk, FAT12_Index far* index, uint16_t cluster, uint8_t far* destination) {
    uint32_t lba = _ClusterToLBA(index, cluster);

    return DISK_Read(disk, lba, g_FATBootRecord->bdb_sectors_per_cluster, (uint8_t far*) destination);
}

uint32_t _DetermineSectorInFAT(uint16_t cluster) {
    return (cluster * 12) / SECTOR_SIZE;
}

uint16_t readFile(DISK* disk, FAT12_Index far* index, FAT12_FILE* file, uint8_t far* destination, uint32_t maxSize, uint32_t* bytesRead) {
    uint32_t readSize = 0;

    if (file->startCluster == file->currentCluster) {
        if (_ReadCluster(disk, index, file->startCluster, destination) != 0) {
            puts("Failed to read first cluster");
            return 3;
        }

        readSize = g_FATBootRecord->bdb_sectors_per_cluster * SECTOR_SIZE;
        destination += (g_FATBootRecord->bdb_sectors_per_cluster) * (SECTOR_SIZE);
    }

    while (readSize < maxSize) {
        uint32_t fatIndex = (file->currentCluster * 3) / 2;
        uint32_t dataCluster;
        if (file->currentCluster % 2 == 0) {
            // It is more "aligned", we get to just read the next 3 nibbles
            dataCluster = (*((uint16_t far*)((uint8_t far*)(index->currentFATSectionBuffer) + fatIndex))) & 0x0FFF;
        } else {
            // We have to shift the data over
            dataCluster = (*((uint16_t far*)((uint8_t far*)(index->currentFATSectionBuffer) + fatIndex))) >> 4;
        }

        if (dataCluster == 0) {
            puts("ERROR: Read cluster of 0");
            return 2;
        }

        if (dataCluster >= 0xFF8) {
            puts("Hit end of cluster chain");
            break;
        }

        if (_ReadCluster(disk, index, dataCluster, destination) != 0) {
            puts("Failed to read file cluster");
            return 1;
        }

        readSize += g_FATBootRecord->bdb_sectors_per_cluster * SECTOR_SIZE;
        destination += g_FATBootRecord->bdb_sectors_per_cluster * SECTOR_SIZE;

        file->currentCluster = dataCluster;
    }

    *bytesRead = readSize;

    return 0;
}
