#include "elf.h"
#include "bio.h"
#include "string.h"
#include "math.h"
#include "long_mode.h"

#define ELF_FILE_MAGIC           0x464C457F
#define ELF_FILE_32BIT           0x01
#define ELF_FILE_64BIT           0x02
#define ELF_FILE_LITTLE_ENDIAN   0x01
#define ET_NONE                  0x00
#define ET_REL                   0x01
#define ET_EXEC                  0x02
#define ET_DYN                   0x03
#define ET_CORE                  0x04
#define X86_64_INSTRUCTION_SET   0x3E

#define PT_NULL                  0x00
#define PT_LOAD                  0x01
#define PT_DYNAMIC               0x02
#define PT_INTERP                0x03
#define PT_NOTE                  0x04
#define PT_SHLIB                 0x05
#define PT_PHDR                  0x06
#define PT_TLS                   0x07
#define PT_GNU_STACK             0x6474e551
#define PT_GNU_EH_FRAME          0x6474e550

#define PF_X                     0x01
#define PF_W                     0x02
#define PF_R                     0x04

static ELF64Header* g_ELF;

typedef struct {
    uint32_t segmentType;
    uint32_t flags;
    uint64_t offset;
    uint64_t virtualAddress;
    uint64_t physicalAddress;
    uint64_t sizeInFile;
    uint64_t sizeInMemory;
    uint64_t alignment;
} __attribute__((packed)) ELF64ProgramHeaderEntry;

void _PrintProgramHeaderTable(uint64_t tableOffset, uint16_t numEntries);

uint16_t readELF(uint8_t* fileBuffer) {
    g_ELF = (ELF64Header*) fileBuffer;

    if (g_ELF->magic != ELF_FILE_MAGIC) {
        puts("File read is not an ELF file");
        return 1;
    }

    if (g_ELF->bitFormat == ELF_FILE_32BIT) {
        puts("32 bit, cannot read");
        return 2;
    } else if (g_ELF->bitFormat != ELF_FILE_64BIT) {
        puts("Unknown ELF file bit format");
        return 3;
    }

    if (g_ELF->endianness != ELF_FILE_LITTLE_ENDIAN) {
        puts("I don't want to support big-endian ELF reading");
        return 4;
    }

    switch (g_ELF->fileType) {
        case ET_EXEC:
            break;
        case ET_REL:
            puts("Relocatable ELF file found");
            return 5;
        case ET_DYN:
            puts("Shared object file found");
            return 5;
        case ET_NONE:
            puts("Unknown ELF file type found");
            return 5;
        case ET_CORE:
            puts("What the heck is an ELF core file?");
            return 5;
        default:
            puts("Actually known ELF file type");
            return 5;
    }

    if (g_ELF->instructionSet != X86_64_INSTRUCTION_SET) {
        puts("Non x86_64 ELF file");
        return 6;
    }

    printf("Program entry point: 0x");
    phexuint64(g_ELF->entryPoint);
    putc('\n');

    uint64_t ph_offset = g_ELF->programHeaderTableOffset;
    uint16_t phnum = g_ELF->programHeaderTableNumEntries;

    _PrintProgramHeaderTable(ph_offset, phnum);

    return 0;
}

void _LoadProgramHeaderEntry(ELF64ProgramHeaderEntry* entry);

void loadAndExecuteELF(uint8_t* fileBuffer) {
    g_ELF = (ELF64Header*) fileBuffer;
    uint64_t entryPoint = g_ELF->entryPoint;
    uint64_t ph_offset = g_ELF->programHeaderTableOffset;
    uint16_t phnum = g_ELF->programHeaderTableNumEntries;

    ELF64ProgramHeaderEntry* table = (ELF64ProgramHeaderEntry*)(((uint8_t*)g_ELF) + ph_offset);

    for (uint16_t i = 0; i < phnum; i++) {
        // If this segment is a loadable segment
        if (table->segmentType == PT_LOAD) {
            _LoadProgramHeaderEntry(table);
        }
        table++;
    }

    puts("Loaded segments into memory");

    puts("Jumping into long mode");

    long_mode_jump((void*)(entryPoint));

    for (;;) {}
}

void _LoadProgramHeaderEntry(ELF64ProgramHeaderEntry* entry) {
    uint64_t offset = entry->offset;
    uint64_t virtualAddress = entry->virtualAddress;
    uint64_t sizeInFile = entry->sizeInFile;
    uint64_t sizeInMemory = entry->sizeInMemory;

    uint8_t* startOfFile = (uint8_t*) g_ELF;
    uint8_t* segmentPtr = startOfFile + offset;
    uint8_t* destination = (uint8_t*) virtualAddress;

    memcpy(segmentPtr, destination, sizeInFile);

    uint64_t bytesToZero = sizeInMemory - sizeInFile;
    uint8_t* bytesToZeroStart = destination + sizeInFile;

    memset(bytesToZeroStart, 0, bytesToZero);

    printf("Loaded 0x");
    phexuint64(sizeInFile);
    printf(" bytes into memory at address 0x");
    phexuint64(virtualAddress);
    putc('\n');

    printf("Zeroed 0x");
    phexuint64(bytesToZero);
    printf(" remaining bytes starting from address ");
    phexuint64((uint64_t)(uint32_t)bytesToZeroStart);
    putc('\n');
}

void _PrintProgramHeaderEntry(ELF64ProgramHeaderEntry* entry);

void _PrintProgramHeaderTable(uint64_t tableOffset, uint16_t numEntries) {
    puts("Program header: ");

    ELF64ProgramHeaderEntry* table = (ELF64ProgramHeaderEntry*)(((uint8_t*)g_ELF) + tableOffset);

    for (uint16_t i = 0; i < numEntries; i++) {
        _PrintProgramHeaderEntry(table);
        table++;
    }
}

const char* _HeaderTypeToStr(uint32_t headerType) {
    switch (headerType) {
        case PT_NULL:
            return "PT_NULL";
        case PT_LOAD:
            return "PT_LOAD";
        case PT_DYNAMIC:
            return "PT_DYNAMIC";
        case PT_INTERP:
            return "PT_INTERP";
        case PT_NOTE:
            return "PT_NOTE";
        case PT_SHLIB:
            return "PT_SHLIB";
        case PT_PHDR:
            return "PT_PHDR";
        case PT_TLS:
            return "PT_TLS";
        case PT_GNU_EH_FRAME:
            return "PT_GNU_EH_FRAME";
        case PT_GNU_STACK:
            return "PT_GNU_STACK";
        default:
            return "UNKNOWN";
    }
}

void _PrintProgramHeaderEntry(ELF64ProgramHeaderEntry* entry) {
    const char* typeStr = _HeaderTypeToStr(entry->segmentType);
    printf("    ");
    printf(typeStr);
    putc('\n');

    printf("  offset ");
    phexuint64(entry->offset);
    putc(' ');

    printf("vaddr ");
    phexuint64(entry->virtualAddress);
    putc(' ');

    printf("paddr ");
    phexuint64(entry->physicalAddress);
    putc(' ');

    putc('\n');

    printf("  filesz ");
    phexuint64(entry->sizeInFile);
    putc(' ');

    printf("memsz ");
    phexuint64(entry->sizeInMemory);
    putc(' ');

    printf("align 2**");
    phexuint8(log2(entry->alignment));
    putc(' ');

    printf("flags ");
    if ((entry->flags & PF_R) != 0) {
        putc('r');
    } else {
        putc('-');
    }
    if ((entry->flags & PF_W) != 0) {
        putc('w');
    } else {
        putc('-');
    }
    if ((entry->flags & PF_X) != 0) {
        putc('x');
    } else {
        putc('-');
    }
    putc('\n');
}
