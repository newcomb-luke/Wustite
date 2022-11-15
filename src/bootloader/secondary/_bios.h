#pragma once

#include "stdint.h"

void _cdecl _BIOS_Video_WriteCharTeletype(char c, uint8_t page);
void _cdecl _BIOS_Video_SetVideoMode(uint8_t mode);
uint16_t _cdecl _BIOS_Drive_Reset(uint8_t drive);
uint16_t _cdecl _BIOS_Drive_GetParams(uint8_t driveNumber,
                                      uint8_t* driveType,
                                      uint8_t* maxHeadOut,
                                      uint16_t* maxCylinderOut,
                                      uint8_t* maxSectorOut);
uint16_t _cdecl _BIOS_Drive_ReadSectors(uint8_t driveNumber,
                                        uint8_t head,
                                        uint16_t cylinder,
                                        uint8_t sector,
                                        uint8_t sectorCount,
                                        uint8_t far* dataDestination);

void _cdecl _x86_div64_32(uint64_t dividend,
                          uint32_t divisor,
                          uint64_t* quotientOut,
                          uint32_t* remainderOut);
