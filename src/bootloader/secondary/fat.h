#pragma once

#include <stdint.h>
#include "bdisk.h"

typedef struct {
    uint8_t bdb_boot_jump[3];
    uint8_t bdb_oem_id[8];
    uint16_t bdb_bytes_per_sector;
    uint8_t bdb_sectors_per_cluster;
    uint16_t bdb_reserved_sectors;
    uint8_t bdb_fat_count;
    uint16_t bdb_dir_entries_count;
    uint16_t bdb_total_sectors;
    uint8_t bdb_media_descriptor_type;
    uint16_t bdb_sectors_per_fat;
    uint16_t bdb_sectors_per_track;
    uint16_t bdb_head_count;
    uint32_t bdb_hidden_sectors;
    uint32_t bdb_large_sectors;

    uint8_t ebr_drive_number;
    uint8_t __reserved;
    uint8_t ebr_signature;
    uint8_t ebr_volume_id[4];
    uint8_t ebr_volume_label[11];
    uint8_t ebr_system_id[8];

    // code and magic number
} __attribute__((packed)) FAT12_BootRecord;

typedef struct {
    uint8_t entryName[8];
    uint8_t entryExt[3];
    uint8_t attributes;
    uint8_t _reserved;
    uint8_t creationTimeTenths;
    uint16_t creationTime;
    uint16_t creationDate;
    uint16_t lastAccessedDate;
    uint16_t firstClusterHigh;
    uint16_t lastModificationTime;
    uint16_t lastModificationDate;
    uint16_t firstClusterLow;
    uint32_t fileSize;
} __attribute__((packed)) FAT12_DirEntry;

typedef struct {
    uint16_t FATStartSector;
    uint16_t rootDirStartSector;
    uint16_t dataRegionStartSector;
    FAT12_DirEntry* currentDirectoryBuffer;
    uint16_t currentDirectoryBufferStartSector;
    uint8_t* currentFATSectionBuffer;
} FAT12_Index;

typedef struct {
    uint16_t startCluster;
    uint16_t currentCluster;
    uint32_t size;
} FAT12_FILE;

uint16_t FAT_DRIVER_INIT(DISK* disk, FAT12_Index* index, uint8_t* currentDirectoryBuffer, uint8_t* currentFATSectionBuffer);

void readOEM(char* buffer);

void readVolumeLabel(char* buffer);

uint16_t openFile(DISK* disk, FAT12_Index* index, FAT12_FILE* fileOut, const char* fileName);

uint16_t readFile(DISK* disk, FAT12_Index* index, FAT12_FILE* file, uint8_t* destination, uint32_t maxSize, uint32_t* bytesRead);
