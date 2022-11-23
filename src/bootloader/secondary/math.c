#include "math.h"

uint64_t log2(uint64_t value) {
    uint64_t log = 0;

    while (value != 0) {
        log += 1;
        value = value >> 1;
    }

    return log;
}

int32_t min(int32_t v1, int32_t v2) {
    if (v1 < v2) {
        return v1;
    }
    return v2;
}

int32_t max(int32_t v1, int32_t v2) {
    if (v1 > v2) {
        return v1;
    }
    return v2;
}