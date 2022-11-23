#include "smap.h"
#include "_bios.h"
#include "bio.h"

void detectMemory(uint32_t* count, SMAPEntry* entryTable) {
    SMAPEntry buffer;
    uint32_t continuation = 0;
    uint32_t bytesRead;
    *count = 0;

    bytesRead = _BIOS_Memory_GetNextSegment(&buffer, &continuation);
    *(entryTable + *count) = buffer;
    (*count)++;

    printf("Entry base: 0x");
    phexuint32(buffer.baseHigh);
    phexuint32(buffer.baseLow);
    printf(", length: 0x");
    phexuint32(buffer.lengthHigh);
    phexuint32(buffer.lengthLow);
    printf(", type: 0x");
    phexuint32(buffer.type);
    putc('\n');

    while (bytesRead > 0 && continuation != 0) {
        bytesRead = _BIOS_Memory_GetNextSegment(&buffer, &continuation);
        printf("Entry base: 0x");
        phexuint32(buffer.baseHigh);
        phexuint32(buffer.baseLow);
        printf(", length: 0x");
        phexuint32(buffer.lengthHigh);
        phexuint32(buffer.lengthLow);
        printf(", type: 0x");
        phexuint32(buffer.type);
        putc('\n');
        // Only these types are valid
        if (buffer.type > 0 && buffer.type < 6) {
            *(entryTable + *count) = buffer;
            (*count)++;
        }
    }
}
