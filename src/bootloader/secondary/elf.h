#pragma once

#include "stdint.h"

#pragma pack(push, 1)
typedef struct {
    uint32_t magic;
    uint8_t bitFormat;
    uint8_t endianness;
    uint8_t headerVersion;
    uint8_t OSABI;
    uint8_t ABIVersion;
    uint8_t _padding[7];
    uint16_t fileType;
    uint16_t instructionSet;
    uint32_t elfVersion;
    uint64_t entryPoint;
    uint64_t programHeaderTableOffset;
    uint64_t sectionHeaderTableOffset;
    uint32_t flags;
    uint16_t headerSize;
    uint16_t programHeaderTableEntrySize;
    uint16_t programHeaderTableNumEntries;
    uint16_t sectionHeaderTableEntrySize;
    uint16_t sectionHeaderTableNumEntries;
    uint16_t sectionHeaderStringTableIndex;
} ELF64Header;
#pragma pack(pop)

uint16_t readELF(uint8_t far* fileBuffer);