#pragma once

#include <stdint.h>

typedef struct {
    uint32_t baseLow;
    uint32_t baseHigh;
    uint32_t lengthLow;
    uint32_t lengthHigh;
    uint32_t type;
    uint32_t ACPI;
} __attribute__((packed)) SMAPEntry;

typedef enum {
    SMAP_USABLE = 1,
    SMAP_RESERVED = 2,
    SMAP_ACPI_RECLAIMABLE = 3,
    SMAP_ACPI_NVS = 4,
    SMAP_BAD_MEMORY = 5
} SMAPEntryType;

void detectMemory(uint32_t* count, SMAPEntry* entryTable);