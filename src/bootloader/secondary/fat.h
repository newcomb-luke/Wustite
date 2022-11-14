#pragma once

#include "stdint.h"

void initFAT(char far* fileStart);

void readOEM(char far* buffer);

void readVolumeLabel(char far* buffer);

void lba_to_chs(uint16_t lba);
