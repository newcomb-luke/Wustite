#pragma once

#include "stdint.h"

void memcpy(const void far* srcptr, void far* dstptr, uint16_t len);

void memset(void far* ptr, uint8_t value, uint16_t len);

int16_t memcmp(const void far* left, const void far* right, uint16_t len);

uint16_t strncpy(const char* srcptr, char* dstptr, uint16_t num);

const char* strchr(const char* str, char chr);

uint16_t strlen(const char* str);
