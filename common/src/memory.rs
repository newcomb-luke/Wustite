#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryRegionType {
    Reserved,
    ACPIReclaimable,
    BootloaderAllocated,
    Usable,
}

// Region type precedence:
//      Reserved >
//      ACPIReclaimable >
//      BootloaderAllocated >
//      Usable

/// This may seem like nonsense, but this allows us to define the "priority"
/// of how the memory regions are unified in the bootloader. If a MemoryRegionType
/// is "greater" than another, then it has a higher priority, and will shrink "lower"
/// types.
///
/// Also I know about derive, but I don't care
///
impl PartialOrd for MemoryRegionType {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(match self {
            Self::Reserved => match other {
                Self::Reserved => core::cmp::Ordering::Equal,
                _ => core::cmp::Ordering::Greater,
            },
            Self::ACPIReclaimable => match other {
                Self::Reserved => core::cmp::Ordering::Less,
                Self::ACPIReclaimable => core::cmp::Ordering::Equal,
                _ => core::cmp::Ordering::Greater,
            },
            Self::BootloaderAllocated => match other {
                Self::Reserved | Self::ACPIReclaimable => core::cmp::Ordering::Less,
                Self::BootloaderAllocated => core::cmp::Ordering::Equal,
                _ => core::cmp::Ordering::Greater,
            },
            Self::Usable => match other {
                Self::Usable => core::cmp::Ordering::Equal,
                _ => core::cmp::Ordering::Less,
            },
        })
    }
}

/// Memory regions supplied by the bootloader are *always* 4 KiB page aligned
/// both the start and end addresses. BootloaderAllocated regions can be reclaimed
/// by the kernel (after they have been read, as they contain data like the memory
/// regions). They will be in sorted order.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct MemoryRegion {
    pub start_addr: u64,
    pub end_addr: u64,
    pub ty: MemoryRegionType,
}

impl MemoryRegion {
    /// Returns true if this region overlaps with another region, allowing
    /// other.start_addr == self.end_addr and self.start_addr == end.end_addr
    /// due to ranges having exclusive ends
    pub fn overlaps_with(&self, other: &Self) -> bool {
        // Deals with if this region overlaps with the end of the other region
        (self.start_addr < other.end_addr && self.end_addr >= other.end_addr)
        // Deals with if this region overlaps with the start of the other region
            || (self.end_addr > other.start_addr && self.start_addr <= other.start_addr)
    }
}
