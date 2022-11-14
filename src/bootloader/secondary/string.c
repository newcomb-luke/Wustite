#include "string.h"

void memcpy(const void far* srcptr, void far* dstptr, uint16_t len) {
    const unsigned char far* src = (const unsigned char far*) srcptr;
    unsigned char far* dst = (unsigned char far*) dstptr;

    for (uint16_t i = 0; i < len; i++) {
        dst[i] = src[i];
    }
}
