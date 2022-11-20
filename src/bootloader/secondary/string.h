#pragma once

#include <stdint.h>

void memcpy(const void* srcptr, void* dstptr, uint32_t len);

void memset(void* ptr, uint8_t value, uint32_t len);

int32_t memcmp(const void* left, const void* right, uint32_t len);

uint32_t strncpy(const char* srcptr, char* dstptr, uint32_t num);

const char* strnchr(const char* str, char chr, uint32_t num);

uint32_t strlen(const char* str);
