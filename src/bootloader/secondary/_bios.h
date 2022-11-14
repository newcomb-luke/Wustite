#pragma once

#include "stdint.h"

void _cdecl _BIOS_Video_WriteCharTeletype(char c, uint8_t page);
void _cdecl _BIOS_Video_SetVideoMode(uint8_t mode);
