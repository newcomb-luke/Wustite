#include "string.h"
#include <stddef.h>

void memcpy(const void* srcptr, void* dstptr, uint32_t len) {
    const uint8_t* src = (const uint8_t*) srcptr;
    uint8_t* dst = (uint8_t*) dstptr;

    for (uint32_t i = 0; i < len; i++) {
        dst[i] = src[i];
    }
}

void memset(void* ptr, uint8_t value, uint32_t len) {
    uint8_t* dst = (uint8_t*) ptr;

    for (uint32_t i = 0; i < len; i++) {
        dst[i] = value;
    }
}

int32_t memcmp(const void* left, const void* right, uint32_t len) {
    const char* leftPtr = (const char*) left;
    const char* rightPtr = (const char*) right;
    int32_t value = 0;

    for (uint32_t i = 0; i < len; i++) {
        value += leftPtr[i] - rightPtr[i];
    }

    return value;
}

// Copies the first num characters of source to destination.
// If the end of the source C string (which is signaled by a null-character)
// is found before num characters have been copied,
// destination has a null character set in that location, and the function returns.
//
// A null-character is implicitly appended at the end of destination if source is
//
// Destination and source shall not overlap.
//
// Returns the number of characters actually written to dstptr, not including
// a null terminator
uint32_t strncpy(const char* src, char* dst, uint32_t num) {
    uint32_t written = 0;

    if (dst == NULL) {
        return 0;
    }

    if (src == NULL) {
        dst[0] = '\0';
        return 0;
    }

    for (uint32_t i = 0; i < num; i++) {
        *dst = *src;

        written++;

        if (*src == '\0') {
            break;
        }

        src++;
        dst++;
    }

    *dst = '\0';

    return written;
}

const char* strnchr(const char* str, char chr, uint32_t num) {
    if (str == NULL) {
        return NULL;
    }

    for (uint32_t i = 0; i < num; i++) {
        if (*str == chr) {
            return str;
        }
        if (*str == '\0') {
            return NULL;
        }

        str++;
    }

    return NULL;
}

uint32_t strlen(const char* str) {
    uint32_t len = 0;

    for (uint32_t i = 0; i < UINT32_MAX; i++) {
        if (*str == '\0') {
            return len;
        }

        str++;
        len++;
    }

    return 0;
}
