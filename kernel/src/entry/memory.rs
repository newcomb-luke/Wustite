// 0x00000000 - 0x000003FF - 1 KiB long - Real Mode IVT
// 0x00000400 - 0x000004FF - 256 bytes long - BIOS Data Area
// 0x00000500 - 0x00002d00 - Secondary bootloader load location (assuming size of 10 KiB)
// 0x00002d00 - 0x00007BFF - Used for bootloader (stage 1 + stage 2) stack
// 0x00007C00 - 0x00007DFF - 512 bytes - OS Boot Sector
// 0x00007E00 - 0x00009200 - 5 KiB - FAT12 Driver Directory Entry Buffer
// 0x00009200 - 0x0000A600 - 5 KiB - FAT12 Driver File Allocation Table Buffer
// 0x0000A600 - 0x0000F600 - 20 KiB - FAT12 Driver File Read Buffer
// 0x0000F600 - 0x00010000 - Empty space
// 0x00010000 - 0x00020000 - Bootloader initialized page table area
// 0x00020000 - 0x00040000 - Kernel load location (assuming size of 128 KiB)
// 0x00040000 - 0x00070000 - Kernel stack begin
// 0x00070000 - 0x0007FFFF - Stage2->Kernel Data Area
// 0x00080000 - 0x0009FFFF - 128 KiB - Extended BIOS Data Area
// 0x000A0000 - 0x000BFFFF - 128 KiB - Video Display Memory
// 0x000C0000 - 0x000C7FFF - 32 KiB - Video BIOS
// 0x000C8000 - 0x000EFFFF - 160 KiB - BIOS Expansions
// 0x000F0000 - 0x000FFFFF - 64 KiB - Motherboard BIOS
// 0x00100000 - 0x00EFFFFF - 14 MiB - RAM free for use
// 0x00F00000 - 0x00FFFFFF - 1 MiB - Possibly memory-mapped hardware

pub struct MemoryRegion {
    pub start: u64,
    pub end: u64,
    pub kind: MemoryRegionKind,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum MemoryRegionKind {
    // Usable memory for anything your kernel developer heart desires
    Usable,
    // Memory used by the bootloader for the initial page table
    BootloaderPageTables,
    // Memory used by the bootloader for the initial global descriptor table
    BootloaderGDT,
    // Reserved as stated by the system firmware
    Reserved,
    // Bad memory as stated by the system firmware
    BadMemory,
    // Reserved but theoretically reclaimable as stated by the system firmware.
    // Probably just treat this as reserved
    ACPIReclaimable,
    // Non volatile storage used by system firmware
    ACPINonVolatileStorage,
}
