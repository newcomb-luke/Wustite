#include "paging.h"
#include "memory.h"

void identityMap(uint32_t numMegabytes) {
    uint64_t* pageTablesBegin = (uint64_t*) PAGE_TABLES_MEM_START;
    uint32_t numRegularPageTables = numMegabytes / MEGABYTES_PER_PAGE_TABLE;
    // The 3 is because of the PML4T, PDPT, and PDT
    uint32_t totalPageTables = numRegularPageTables + 3;
    uint64_t* pageMapLevel4TableBegin = pageTablesBegin;
    uint64_t* pageDirectoryPointerTableBegin = pageMapLevel4TableBegin + ENTRIES_PER_PAGE_TABLE;
    uint64_t* pageDirectoryTableBegin = pageDirectoryPointerTableBegin + ENTRIES_PER_PAGE_TABLE;
    uint64_t* firstPageTableBegin = pageDirectoryTableBegin + ENTRIES_PER_PAGE_TABLE;

    // Zero all the memory for the page tables first, just in case
    for (uint32_t i = 0; i < totalPageTables; i++) {
        for (uint32_t j = 0; j < ENTRIES_PER_PAGE_TABLE; j++) {
            *(pageTablesBegin + j + i * ENTRIES_PER_PAGE_TABLE) = 0;
        }
    }

    // Set up the first entry of the PML4T, only one entry is all we will need
    // This stores a "pointer" to the page directory pointer table, and sets bits 0 and 1
    // Setting bit 0 sets the entry as present, and bit 1 sets it as writable
    pageMapLevel4TableBegin[0] = PAGE_DIRECTORY_POINTER_TABLE_START + 0b00000011;
    // Set up the first entry of the PDPT, only one entry is all we need
    // Same value as above for the same reasons
    pageDirectoryPointerTableBegin[0] = PAGE_DIRECTORY_TABLE_START + 0b00000011;
    // Here is where it gets a little fun. We need as many entries as numRegularPageTables.
    // This points numRegularPageTables page directory table entries to pointers
    // to the corresponding page tables. Each one resides at PAGE_TABLES_START + PAGE_TABLE_SIZE * 1.
    // That is to say, they all start at PAGE_TABLES_START, offset by however many previous page tables
    // we have already set.
    for (uint32_t i = 0; i < numRegularPageTables; i++) {
        pageDirectoryTableBegin[i] = (PAGE_TABLES_START + (PAGE_TABLE_SIZE * i)) + 0b00000011;
    }
    // Now we need to actually set up our page tables
    // We will start at address 0, and work up by increments of the page size. We
    // use the default size, as defined in memory.h, of 4KiB pages.
    uint64_t currentPhysicalAddress = 0;
    uint64_t* currentPageBegin = firstPageTableBegin;

    for (uint32_t pageTable = 0; pageTable < numRegularPageTables; pageTable++) {
        for (uint32_t entry = 0; entry < ENTRIES_PER_PAGE_TABLE; entry++) {
            // Set the physical address, and writable and present bits
            *(currentPageBegin + entry) = currentPhysicalAddress + 0b00000011;
            currentPhysicalAddress += SIZE_OF_SINGLE_PAGE;
        }
        // Do the next table
        currentPageBegin += ENTRIES_PER_PAGE_TABLE;
    }

    // Now we are done :) hopefully it works
}