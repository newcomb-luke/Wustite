#include "elf.h"
#include "bio.h"
#include "string.h"

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

static ELF64Header far* g_ELF;

#pragma pack(push, 1)
typedef struct {
    uint32_t segmentType;
    uint32_t flags;
    uint64_t offset;
    uint64_t virtualAddress;
    uint64_t physicalAddress;
    uint64_t sizeInFile;
    uint64_t sizeInMemory;
    uint64_t alignment;
} ELF64ProgramHeaderEntry;
#pragma pack(pop)

void _PrintProgramHeaderTable(uint32_t tableOffset, uint32_t numEntries);

uint16_t readELF(uint8_t far* fileBuffer) {
    g_ELF = (ELF64Header far*) fileBuffer;

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
    uint32_t entryPointUpper = *(((uint32_t far*)&g_ELF->entryPoint) + 1);
    uint32_t entryPointLower = *(((uint32_t far*)&g_ELF->entryPoint) + 0);
    phexuint32(entryPointUpper);
    phexuint32(entryPointLower);
    putc('\n');

    // If this value takes the full 64 bits to store, I'll eat a hat made out of gummy bears
    uint32_t ph_offset = *(((uint32_t far*)&g_ELF->programHeaderTableOffset) + 0);
    uint32_t phnum = g_ELF->programHeaderTableNumEntries;

    _PrintProgramHeaderTable(ph_offset, phnum);

    return 0;
}

void _PrintProgramHeaderEntry(ELF64ProgramHeaderEntry far* entry);

void _PrintProgramHeaderTable(uint32_t tableOffset, uint32_t numEntries) {
    puts("Program header: ");

    ELF64ProgramHeaderEntry far* table = (ELF64ProgramHeaderEntry far*)(((uint8_t far*)g_ELF) + tableOffset);

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

uint8_t _AlignmentToU8(uint32_t encoded) {
    if (encoded == 0 || encoded == 1) {
        return 0;
    }

    uint8_t power = 0;

    while (encoded != 0) {
        encoded = encoded >> 4;
        power++;
    }

    return power;
}

void _PrintProgramHeaderEntry(ELF64ProgramHeaderEntry far* entry) {
const char* typeStr = _HeaderTypeToStr(entry->segmentType);
printf("    ");
printf(typeStr);
putc('\n');

printf("  offset ");
uint32_t upper = *(((uint32_t far*)&entry->offset) + 1);
uint32_t lower = *(((uint32_t far*)&entry->offset) + 0);
phexuint32(upper);
phexuint32(lower);
putc(' ');

printf("vaddr ");
upper = *(((uint32_t far*)&entry->virtualAddress) + 1);
lower = *(((uint32_t far*)&entry->virtualAddress) + 0);
phexuint32(upper);
phexuint32(lower);
putc(' ');

printf("paddr ");
upper = *(((uint32_t far*)&entry->physicalAddress) + 1);
lower = *(((uint32_t far*)&entry->physicalAddress) + 0);
phexuint32(upper);
phexuint32(lower);
putc(' ');

putc('\n');

printf("  filesz ");
upper = *(((uint32_t far*)&entry->sizeInFile) + 1);
lower = *(((uint32_t far*)&entry->sizeInFile) + 0);
phexuint32(upper);
phexuint32(lower);
putc(' ');

printf("memsz ");
upper = *(((uint32_t far*)&entry->sizeInMemory) + 1);
lower = *(((uint32_t far*)&entry->sizeInMemory) + 0);
phexuint32(upper);
phexuint32(lower);
putc(' ');

printf("align 2**");
phexuint8(_AlignmentToU8((uint32_t)entry->alignment));
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
