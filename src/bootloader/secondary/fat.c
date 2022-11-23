#include "fat.h"
#include <stdint.h>
#include <stddef.h>
#include "math.h"
#include "memory.h"
#include "string.h"
#include "bio.h"
#include "ctype.h"
#include "bdisk.h"

#define SECTOR_SIZE     512
#define FILE_NAME_LEN   11
#define DIR_ENTRY_SIZE  32

typedef enum {
    READ_ONLY = 0x01,
    HIDDEN = 0x02,
    SYSTEM = 0x04,
    VOLUME_ID = 0x08,
    DIRECTORY = 0x10,
    ARCHIVE = 0x20,
    LFN = READ_ONLY | HIDDEN | SYSTEM | VOLUME_ID
} __attribute__((packed)) DirectoryAttribute;

typedef enum {
    FREE_CLUSTER = 0x000,
    BAD_CLUSTER = 0xFF7,
    LAST_CLUSTER = 0xFF8,
} __attribute__((packed)) FAT12Cluster;

uint16_t _LoadRootDirectory(DISK* disk, FAT12_Index* index);

uint16_t FAT_DRIVER_INIT(DISK* disk,
                         FAT12_Index* index,
                         uint8_t* bootRecordBuffer,
                         uint8_t* directoryBuffer,
                         uint8_t* currentFATSectionBuffer,
                         uint8_t* loadBuffer) {
    index->directoryBuffer = (FAT12_DirEntry*) directoryBuffer;
    index->currentFATSectionBuffer = currentFATSectionBuffer;
    index->bootRecord  = (FAT12_BootRecord*) bootRecordBuffer;
    index->loadBuffer = loadBuffer;

    if (DISK_Read(disk, 0, 1, bootRecordBuffer) != 0) {
        puts("Could not read boot sector");
        return 1;
    }

    index->FATStartSector = index->bootRecord->bdb_reserved_sectors;

    index->rootDirStartSector = index->FATStartSector + index->bootRecord->bdb_fat_count * index->bootRecord->bdb_sectors_per_fat;
    index->rootDirSizeInSectors = ((index->bootRecord->bdb_dir_entries_count * 32) + (index->bootRecord->bdb_bytes_per_sector - 1)) / SECTOR_SIZE;

    // This calculation rounds up to the nearest whole sector, which is how the data is stored if it doesn't fit neatly
    index->dataRegionStartSector = index->rootDirStartSector + index->rootDirSizeInSectors;

    if (_LoadRootDirectory(disk, index) != 0) {
        puts("Loading root directory failed");
        return 2;
    }

    if (DISK_Read(disk, index->FATStartSector, index->bootRecord->bdb_sectors_per_fat, (uint8_t*) index->currentFATSectionBuffer) != 0) {
        puts("Loading FAT failed");
        return 3;
    }

    return 0;
}

uint32_t _ClusterToLBA(FAT12_Index* index, uint16_t cluster) {
    return index->dataRegionStartSector + (cluster - 2) * index->bootRecord->bdb_sectors_per_cluster;
}

void readOEM(FAT12_Index* index, char* buffer) {
    memcpy((void*) &(index->bootRecord->bdb_oem_id), (void*) buffer, 8);
}

void readVolumeLabel(FAT12_Index* index, char* buffer) {
    memcpy((void*) &index->bootRecord->ebr_volume_label, (void*) buffer, 11);
}

uint16_t _LoadSector(DISK* disk, FAT12_Index* index, uint32_t lba, uint8_t* destination) {
    if (DISK_Read(disk, lba, 1, index->loadBuffer)) {
        puts("Failed in _LoadSector DISK_READ");
        return 1;
    }
    memcpy((void*)index->loadBuffer, (void*)destination, SECTOR_SIZE);
    return 0;
}

uint16_t _LoadSectors(DISK* disk, FAT12_Index* index, uint32_t startLBA, uint16_t sectorsToRead, uint8_t* destination) {
    for (uint32_t lba = startLBA; lba < startLBA + sectorsToRead; lba++) {
        if (_LoadSector(disk, index, lba, destination) != 0) {
            puts("Failure in _LoadSectors");
            return 1;
        }
        destination += SECTOR_SIZE;
    }
    return 0;
}

uint16_t _LoadRootDirectory(DISK* disk, FAT12_Index* index) {
    uint16_t sectorsToLoad = min(index->rootDirSizeInSectors, FAT_CURRENT_DIRECTORY_BUFFER_SIZE / SECTOR_SIZE);
    if (_LoadSectors(disk, index, 
                            index->rootDirStartSector, 
                            sectorsToLoad,
                            (uint8_t*)(index->directoryBuffer)) != 0) {
        puts("Failure in _LoadRootDirectory");
        return 1;
    }

    index->directoryBufferStartSector = index->rootDirStartSector;

    return 0;
}

uint16_t _CStrTo8Point3(const char* fileName, char* nameBuffer) {
    uint16_t nameLen = strlen(fileName);

    // size of 12 counts the . that isn't present in the 8.3 filename
    if (nameLen > 12 || nameLen < 1) {
        puts("File name too long or empty");
        return 1;
    }

    const char* sep = strnchr(fileName, '.', FILE_NAME_LEN + 1);

    memset((void*) nameBuffer, ' ', FILE_NAME_LEN);

    if (sep == NULL) {
        if (nameLen > 8) {
            return 2;
        }

        // Just copy the plain name over
        memcpy((void*) fileName, (void*) nameBuffer, nameLen);
    } else {
        uint8_t namePartLen = (uint8_t)(sep - fileName);
        // -1 for the dot before the file extension
        uint8_t extPartLen = nameLen - namePartLen - 1;

        if (namePartLen > 8 || extPartLen > 3) {
            puts("Invalid file name length");
            return 3;
        }

        const char* extPtr = &fileName[namePartLen + 1];

        memcpy((void*) fileName, (void*) nameBuffer, namePartLen);

        memcpy((void*) extPtr, (void*) &nameBuffer[8], extPartLen);
    }

    for (uint8_t i = 0; i < FILE_NAME_LEN; i++) {
        nameBuffer[i] = toupper(nameBuffer[i]);
    }

    return 0;
}

FAT12_DirEntry* _FindEntryInRootDirectory(DISK* disk, FAT12_Index* index, char* name) {
    if (_LoadRootDirectory(disk, index) != 0) {
        puts("Failed to re-load root directory while finding entry");
        return NULL;
    }

    uint16_t currentRootDirStartSector = index->rootDirStartSector;

    uint16_t directoriesPerBuffer = FAT_CURRENT_DIRECTORY_BUFFER_SIZE / DIR_ENTRY_SIZE;
    int16_t rootDirSectorsRemaining = index->rootDirSizeInSectors - FAT_CURRENT_DIRECTORY_BUFFER_SIZE / SECTOR_SIZE;

    for(;;) {
        // Our current root directory buffer can store 9 sectors of
        // information. This means that we can store up to 144 directory
        // entries at one time. If the file we are looking for isn't in
        // the ones we loaded first, we load the second set, if there is one.
        for (int i = 0; i < directoriesPerBuffer; i++) {
            FAT12_DirEntry* entry = (FAT12_DirEntry*) &index->directoryBuffer[i];

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

            if (memcmp((void*) &entry->entryName, (void*) name, FILE_NAME_LEN) == 0) {
                return entry;
            }
        }

        if (rootDirSectorsRemaining < 0) {
            puts("Reading root directory overflowed and did not find end directory entry");
            return NULL;
        }

        // Read the next section of it
        currentRootDirStartSector += min(rootDirSectorsRemaining, FAT_CURRENT_DIRECTORY_BUFFER_SIZE / SECTOR_SIZE);
        rootDirSectorsRemaining -= FAT_CURRENT_DIRECTORY_BUFFER_SIZE / SECTOR_SIZE;

        if (_LoadSectors(disk, index, currentRootDirStartSector, 
                                               FAT_CURRENT_DIRECTORY_BUFFER_SIZE / SECTOR_SIZE,
                                               (uint8_t*) index->directoryBuffer) != 0) {
            puts("Failed to read next section of directory");
            return NULL;
        }
    }

    return NULL;
}

uint16_t openFile(DISK* disk, FAT12_Index* index, FAT12_FILE* fileOut, const char* fileName) {
    char FAT12FileNameBuffer[FILE_NAME_LEN];

    // We will always begin by re-loading the root directory
    if (_LoadRootDirectory(disk, index) != 0) {
        return 1;
    }

    _CStrTo8Point3(fileName, (char*)&FAT12FileNameBuffer);

    FAT12_DirEntry* entry = _FindEntryInRootDirectory(disk, index, (char*)&FAT12FileNameBuffer);

    if (entry == NULL) {
        return 2;
    }

    if ((entry->attributes & DIRECTORY) != 0) {
        puts("Subdirectories are not supported");
        return 3;
    }

    printf("Found ");
    puts(fileName);

    fileOut->startCluster = entry->firstClusterLow;
    fileOut->currentCluster = entry->firstClusterLow;
    fileOut->size = entry->fileSize;

    return 0;
}

uint16_t _ReadCluster(DISK* disk, FAT12_Index* index, uint16_t cluster, uint8_t* destination) {
    uint32_t lba = _ClusterToLBA(index, cluster);

    return _LoadSectors(disk,
                        index,
                        lba,
                        index->bootRecord->bdb_sectors_per_cluster,
                        destination);
}

uint32_t _DetermineSectorInFAT(uint16_t cluster) {
    return (cluster * 12) / SECTOR_SIZE;
}

int32_t _LoadFATSectors(DISK* disk, FAT12_Index* index, uint32_t requestedCluster) {
    uint32_t sector = _DetermineSectorInFAT(requestedCluster);
    uint32_t lba = sector + index->FATStartSector;

    printf("Cluster 0x");
    phexuint32(requestedCluster);
    printf(" exists in FAT sector ");
    phexuint32(sector);
    putc('\n');

    if (_LoadSectors(disk, index, lba, FAT_CURRENT_FAT_SECTION_BUFFER_SIZE / SECTOR_SIZE, index->currentFATSectionBuffer) != 0) {
        return -1;
    }

    return (sector * SECTOR_SIZE) / 12;
}

uint16_t readFile(DISK* disk, FAT12_Index* index, FAT12_FILE* file, uint8_t* destination, uint32_t maxSize, uint32_t* bytesRead) {
    uint32_t readSize = 0;

    uint32_t currentFATBufferStartSector = _DetermineSectorInFAT(file->currentCluster);

    uint32_t fatIndexOffset = _LoadFATSectors(disk, index, file->currentCluster);

    printf("FAT index offset: ");
    phexuint32(fatIndexOffset);
    putc('\n');

    if (file->startCluster == file->currentCluster) {
        if (_ReadCluster(disk, index, file->startCluster, destination) != 0) {
            puts("Failed to read first cluster");
            return 3;
        }

        readSize = index->bootRecord->bdb_sectors_per_cluster * SECTOR_SIZE;
        destination += (index->bootRecord->bdb_sectors_per_cluster) * (SECTOR_SIZE);
    }

    while (readSize < maxSize) {
        uint32_t currentSectorInFAT = _DetermineSectorInFAT(file->currentCluster);
        uint32_t relativeSectorInFATBuffer = currentSectorInFAT - currentFATBufferStartSector;

        if (relativeSectorInFATBuffer >= (FAT_CURRENT_FAT_SECTION_BUFFER_SIZE / SECTOR_SIZE) - 1) {
            printf("NEEDS MORE: Bytes read: 0x");
            phexuint32(readSize);
            putc('\n');

            currentFATBufferStartSector = _DetermineSectorInFAT(file->currentCluster);
            fatIndexOffset = _LoadFATSectors(disk, index, file->currentCluster);

            printf("New sector start: ");
            phexuint32(currentFATBufferStartSector);
            putc('\n');

            printf("FAT index offset: ");
            phexuint32(fatIndexOffset);
            putc('\n');

            printf("TODO!: Implement FAT sector swapping");

            for (;;) {}
        }

        uint32_t fatIndex = (file->currentCluster * 3) / 2;
        uint32_t dataCluster;
        if (file->currentCluster % 2 == 0) {
            // It is more "aligned", we get to just read the next 3 nibbles
            dataCluster = (*((uint16_t*)((uint8_t*)(index->currentFATSectionBuffer) + fatIndex - fatIndexOffset))) & 0x0FFF;
        } else {
            // We have to shift the data over
            dataCluster = (*((uint16_t*)((uint8_t*)(index->currentFATSectionBuffer) + fatIndex - fatIndexOffset))) >> 4;
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

        readSize += index->bootRecord->bdb_sectors_per_cluster * SECTOR_SIZE;
        destination += index->bootRecord->bdb_sectors_per_cluster * SECTOR_SIZE;

        file->currentCluster = dataCluster;
    }

    *bytesRead = readSize;

    return 0;
}
