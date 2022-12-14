#pragma once

#include <stdint.h>

#define UPPER_CONVENTIONAL_START 0x00000500
#define UPPER_CONVENTIONAL_END   0x00007BFF
#define BOOT_SECTOR_START        0x00007C00
#define LOWER_CONVENTIONAL_START 0x00007E00
#define LOWER_CONVENTIONAL_END   0x0007FFFF

#define SECONDARY_BOOTLOADER_START 0x00000500

#define FAT_DRIVER_MEMORY_START               LOWER_CONVENTIONAL_START

#define FAT_BOOT_RECORD_BUFFER_START          FAT_DRIVER_MEMORY_START
#define FAT_BOOT_RECORD_BUFFER_SIZE           (512)
#define FAT_CURRENT_DIRECTORY_BUFFER_START    (uint8_t*)(FAT_BOOT_RECORD_BUFFER_START + FAT_BOOT_RECORD_BUFFER_SIZE)
#define FAT_CURRENT_DIRECTORY_BUFFER_SIZE     (5 * 512)
#define FAT_CURRENT_FAT_SECTION_BUFFER_START  (uint8_t*)(FAT_CURRENT_DIRECTORY_BUFFER_START + FAT_CURRENT_DIRECTORY_BUFFER_SIZE)
#define FAT_CURRENT_FAT_SECTION_BUFFER_SIZE   (14 * 512)
#define FAT_LOAD_BUFFER_START                 (uint8_t*)(FAT_CURRENT_FAT_SECTION_BUFFER_START + FAT_CURRENT_FAT_SECTION_BUFFER_SIZE)
#define FAT_LOAD_BUFFER_SIZE                  (512)

#define PAGE_TABLE_SIZE                           0x1000
#define PAGE_TABLES_MEM_START                     0x00010000
#define PAGE_MAP_LEVEL_4_TABLE_START              (PAGE_TABLES_MEM_START)
#define PAGE_DIRECTORY_POINTER_TABLE_START        (PAGE_MAP_LEVEL_4_TABLE_START + PAGE_TABLE_SIZE)
#define PAGE_DIRECTORY_TABLE_START                (PAGE_DIRECTORY_POINTER_TABLE_START + PAGE_TABLE_SIZE)
#define PAGE_TABLES_START                         (PAGE_DIRECTORY_TABLE_START + PAGE_TABLE_SIZE)
#define NUM_MEGABYTES_TO_MAP                      8
#define MEGABYTES_PER_PAGE_TABLE                  2
#define ENTRIES_PER_PAGE_TABLE                    512
#define SIZE_OF_SINGLE_PAGE                       4096

#define BOOT_DRIVE_MEM_LOC                        0x70000
#define SMAP_ENTRIES_MEM_START                    BOOT_DRIVE_MEM_LOC + 4
#define SMAP_NUM_ENTRIES_LOC                      SMAP_ENTRIES_MEM_START
#define SMAP_FIRST_ENTRY_LOC                      SMAP_NUM_ENTRIES_LOC + 4
#define MAX_SMAP_ENTRIES                          2730

#define KERNEL_READ_LOCATION_START                0x00020000
#define KERNEL_READ_LOCATION_SIZE                 0x00050000

#define KERNEL_STACK_TOP                          0x00200000
#define KERNEL_STACK_BOTTOM                       0x00300000

// Target memory map:
// 0x00000000 - 0x000003FF - 1 KiB long - Real Mode IVT
// 0x00000400 - 0x000004FF - 256 bytes long - BIOS Data Area
// 0x00000500 - 0x00007BFF - "Almost 30 KiB" - Conventional memory
// 0x00007C00 - 0x00007DFF - 512 bytes - OS Boot Sector
// 0x00007E00 - 0x0007FFFF - 480.5 KiB - Conventional memory
// 0x00080000 - 0x0009FFFF - 128 KiB - Extended BIOS Data Area
// 0x000A0000 - 0x000BFFFF - 128 KiB - Video Display Memory
// 0x000C0000 - 0x000C7FFF - 32 KiB - Video BIOS
// 0x000C8000 - 0x000EFFFF - 160 KiB - BIOS Expansions
// 0x000F0000 - 0x000FFFFF - 64 KiB - Motherboard BIOS
// 0x00100000 - 0x00EFFFFF - 14 MiB - RAM free for use
// 0x00F00000 - 0x00FFFFFF - 1 MiB - Possibly memory-mapped hardware
// 0x01000000 - ?????????? - RAM free for use
// 0xC0000000 - 0xFFFFFFFF - 1 GiB - Typically reserved for memory-mapped hardware and other stuff
// 0x0000000100000000 - ?????????????????? - RAM free for use

// During stage 1:
// 0x00000500 - 0x00002d00 - Secondary bootloader load location (assuming size of 10 KiB)
// 0x00002d00 - 0x00007BFF - Used for bootloader (stage 1 + stage 2) stack
// 0x00007C00 - 0x00007DFF - 512 bytes - OS Boot Sector
// 0x00007E00 - 0x00008000 - 512 bytes, used as buffer for reading FAT12
// During stage 2:
// 0x00000500 - 0x00002d00 - Secondary bootloader load location (assuming size of 10 KiB)
// 0x00002d00 - 0x00007BFF - Used for bootloader (stage 1 + stage 2) stack
// 0x00007C00 - 0x00007DFF - 512 bytes - OS Boot Sector
// 0x00007E00 - 0x00008000 - 512 bytes - FAT12 Driver Boot Record Buffer
// 0x00008000 - 0x00009200 - 5 KiB - FAT12 Driver Directory Entry Buffer
// 0x00009200 - 0x0000A600 - 5 KiB - FAT12 Driver File Allocation Table Buffer
// 0x0000A600 - 0x0000F600 - 20 KiB - FAT12 Driver File Read Buffer
// 0x0000F600 - 0x00010000 - Empty space
// 0x00010000 - 0x00020000 - Bootloader initialized page table area
// 0x00020000 - 0x00070000 - Kernel read location (maximum size of 320 KiB)
// 0x00070000 - 0x0007FFFF - Stage2->Kernel Data Area
// 0x00080000 - 0x0009FFFF - 128 KiB - Extended BIOS Data Area
// 0x000A0000 - 0x000BFFFF - 128 KiB - Video Display Memory
// 0x000C0000 - 0x000C7FFF - 32 KiB - Video BIOS
// 0x000C8000 - 0x000EFFFF - 160 KiB - BIOS Expansions
// 0x000F0000 - 0x000FFFFF - 64 KiB - Motherboard BIOS
// 0x00100000 - 0x00200000 - Kernel Location
// 0x00200000 - 0x00300000 - Kernel Stack Region
// 0x00300000 - 0x00301000 - Kernel PML4T
// 0x00301000 - 0x00302000 - Kernel Page Directory Pointer Table
// 0x00302000 - 0x00303000 - Kernel Page Directory Table
// 0x00303000 - 0x0030b000 - Kernel Page Tables
// 0x0030b000 - 0x00EFFFFF - 12 MiB - RAM free for use
// 0x00F00000 - 0x00FFFFFF - 1 MiB - Possibly memory-mapped hardware