#include "string.h"

void memcpy(const void far* srcptr, void far* dstptr, uint16_t len) {
    const uint8_t far* src = (const uint8_t far*) srcptr;
    uint8_t far* dst = (uint8_t far*) dstptr;

    for (uint16_t i = 0; i < len; i++) {
        dst[i] = src[i];
    }
}

void memset(void far* ptr, uint8_t value, uint16_t len) {
    uint8_t far* dst = (uint8_t far*) ptr;

    for (uint16_t i = 0; i < len; i++) {
        dst[i] = value;
    }
}

int16_t memcmp(const void far* left, const void far* right, uint16_t len) {
    const char far* leftPtr = (const char far*) left;
    const char far* rightPtr = (const char far*) right;
    int16_t value = 0;

    for (uint16_t i = 0; i < len; i++) {
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
uint16_t strncpy(const char* src, char* dst, uint16_t num) {
    uint16_t written = 0;

    if (dst == NULL) {
        return 0;
    }

    if (src == NULL) {
        dst[0] = '\0';
        return 0;
    }

    for (uint16_t i = 0; i < num; i++) {
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

const char* strnchr(const char* str, char chr, uint16_t num) {
    if (str == NULL) {
        return NULL;
    }

    for (uint16_t i = 0; i < num; i++) {
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

uint16_t strlen(const char* str) {
    uint16_t len = 0;

    for (uint16_t i = 0; i < UINT16_MAX; i++) {
        if (*str == '\0') {
            return len;
        }

        str++;
        len++;
    }

    return 0;
}
