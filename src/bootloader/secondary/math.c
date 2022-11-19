#include "math.h"

uint64_t log2(uint64_t value) {
    uint64_t log = 0;

    while (value != 0) {
        log += 1;
        value = value >> 1;
    }

    return log;
}
