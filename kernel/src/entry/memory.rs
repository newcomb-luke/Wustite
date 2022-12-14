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
