#pragma once

#include "stdint.h"

void initFAT(char far* fileStart);

void readOEM(char far* buffer);

void readVolumeLabel(char far* buffer);
