#pragma once

#include <stdint.h>

void memcpy(const void* srcptr, void* dstptr, uint16_t len);

void memset(void* ptr, uint8_t value, uint16_t len);

int16_t memcmp(const void* left, const void* right, uint16_t len);

uint16_t strncpy(const char* srcptr, char* dstptr, uint16_t num);

const char* strnchr(const char* str, char chr, uint16_t num);

uint16_t strlen(const char* str);
