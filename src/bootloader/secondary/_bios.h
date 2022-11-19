#pragma once

#include <stdint.h>

void __attribute__((cdecl)) _BIOS_Video_WriteCharTeletype(char c);
void __attribute__((cdecl)) _BIOS_Video_SetVideoMode(uint8_t mode);
uint16_t __attribute__((cdecl)) _BIOS_Drive_Reset(uint8_t drive);
uint16_t __attribute__((cdecl)) _BIOS_Drive_GetParams(uint8_t driveNumber,
                                      uint8_t* driveType,
                                      uint8_t* maxHeadOut,
                                      uint16_t* maxCylinderOut,
                                      uint8_t* maxSectorOut);
uint16_t __attribute__((cdecl)) _BIOS_Drive_ReadSectors(uint8_t driveNumber,
                                        uint8_t head,
                                        uint16_t cylinder,
                                        uint8_t sector,
                                        uint8_t sectorCount,
                                        uint8_t* dataDestination);
