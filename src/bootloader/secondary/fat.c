#include "fat.h"
#include "stdint.h"
#include "string.h"

#pragma pack(push, 1)
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
} FAT12_BootRecord;
#pragma pack(pop)

static FAT12_BootRecord far* g_currentFATBR;

void initFAT(char far* fileStart) {
    g_currentFATBR = (FAT12_BootRecord far*) fileStart;
}

void readOEM(char far* buffer) {
    memcpy((void far*) &g_currentFATBR->bdb_oem_id, (void far*) buffer, 8);
}

void readVolumeLabel(char far* buffer) {
    memcpy((void far*) &g_currentFATBR->ebr_volume_label, (void far*) buffer, 11);
}

void lba_to_chs(uint16_t lba) {

}

