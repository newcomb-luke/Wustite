use core::mem::MaybeUninit;

use uefi::{
    prelude::BootServices,
    table::boot::{MemoryDescriptor, MemoryMap, MemoryType},
};

use common::memory::{MemoryRegion, MemoryRegionType};
use uefi_services::{print, println};

/// To make our lives easier (no reallocation) we are going to just allocate
/// a fixed number of spots for unified memory regions and hope for the best.
const MAX_NUM_REGIONS: usize = 256;

/// UEFI Spec: "All of lower memory is reported as normal memory. The OS
/// must handle standard RAM locations that are reserved for specific uses,
/// such as the interrupt vector table (0:0) and the platform boot firmware
/// data area (40:0). To preserve backward compatibility, platform should
/// avoid using persistent memory to materialize the lower memory.
/// If persistent memory is used for lower memory, platform boot
/// firmware must report the lower memory address range using
/// AddressRangeMemory and must not report using AddressRangePersistentMemory.
const IGNORE_BEFORE: u64 = 0x100000;

/// Gets the memory map from the firmware and allocates a spot in memory for it.
///
/// This function maps the UEFI memory map entries to our own MemoryRegions, and returns
/// a pointer to the first one, along with the number of regions. They are sorted,
/// and "unified" (no overlapping regions, adjacent regions are merged).
///
pub fn get_memory_map(
    boot_services: &BootServices,
) -> Result<(*const MemoryRegion, u64), GetMemoryMapError> {
    let before_size = boot_services.memory_map_size();

    // We want to anticipate a few extra entries here, because of our allocation
    // to store the actual map
    // WORST case is 10 new entries I'd imagine, (QEMU, I'm looking at you)
    const FUDGE_AMOUNT: usize = 10;

    let allocation_size = before_size.map_size + before_size.entry_size * FUDGE_AMOUNT;

    let buffer_start = boot_services.allocate_pool(MemoryType::LOADER_DATA, allocation_size)?;

    let buffer = unsafe { core::slice::from_raw_parts_mut(buffer_start, allocation_size) };

    let mut uefi_memory_map = boot_services.memory_map(buffer)?;

    // Nice.
    uefi_memory_map.sort();

    convert_map(boot_services, uefi_memory_map)
}

fn convert_map(
    boot_services: &BootServices,
    uefi_memory_map: MemoryMap,
) -> Result<(*const MemoryRegion, u64), GetMemoryMapError> {
    let regions_allocation_size = MAX_NUM_REGIONS * core::mem::size_of::<MemoryRegion>();

    let regions_buffer_start =
        boot_services.allocate_pool(MemoryType::LOADER_DATA, regions_allocation_size)?;

    let mut regions = MemoryRegions::new(regions_buffer_start as *mut MemoryRegion);

    let mut unified = StackMemoryRegions::new();

    for entry in uefi_memory_map.entries() {
        let converted = MemoryRegion {
            start_addr: entry.phys_start,
            end_addr: entry.end(),
            ty: entry.ty(),
        };

        unified.constrain(&uefi_memory_map, converted)?;
    }

    regions.extend_from(unified.entries())?;

    println!("Memory map: ");

    for entry in regions.entries() {
        println!(
            "{:08x} -> {:08x} : {:?}",
            entry.start_addr, entry.end_addr, entry.ty
        );
    }

    for entry in regions.entries() {
        for other in regions.entries() {
            if entry != other {
                assert!(!entry.overlaps_with(other));
            }
        }
    }

    Ok(regions.into_parts())
}

#[derive(Debug, Clone)]
pub enum GetMemoryMapError {
    TooManyMemoryRegionsErrror,
    UEFIError(uefi::Error),
}

impl From<uefi::Error> for GetMemoryMapError {
    fn from(value: uefi::Error) -> Self {
        Self::UEFIError(value)
    }
}

struct StackMemoryRegions {
    regions: [MaybeUninit<MemoryRegion>; MAX_NUM_REGIONS],
    num_regions: usize,
}

impl StackMemoryRegions {
    fn new() -> Self {
        Self {
            regions: [MaybeUninit::uninit(); MAX_NUM_REGIONS],
            num_regions: 0,
        }
    }

    fn add_region(&mut self, region: MemoryRegion) -> Result<(), GetMemoryMapError> {
        if self.num_regions >= self.regions.len() {
            return Err(GetMemoryMapError::TooManyMemoryRegionsErrror);
        }

        unsafe {
            self.regions
                .get_unchecked_mut(self.num_regions)
                .write(region);
        }

        self.num_regions += 1;

        Ok(())
    }

    fn constrain(
        &mut self,
        map: &MemoryMap,
        mut region: MemoryRegion,
    ) -> Result<(), GetMemoryMapError> {
        // See if this starts before 1 MiB (the ignore area)
        if region.start_addr < IGNORE_BEFORE {
            region.start_addr = IGNORE_BEFORE;
        }

        if region.start_addr >= region.end_addr {
            return Ok(());
        }

        // See if this entry is already covered by another region of >= type, and if so, don't add
        // it
        for already in self.entries() {
            if already.start_addr <= region.start_addr
                && already.end_addr >= region.end_addr
                && already.ty >= region.ty
            {
                return Ok(());
            }
        }

        // Takes care of non-compatible regions overlapping on the left of this region
        for other in map.entries() {
            if other.phys_start <= region.start_addr && other.end() > region.start_addr {
                if other.ty() > region.ty {
                    region.start_addr = region.start_addr.max(other.end());
                }
            }
        }

        if region.start_addr >= region.end_addr {
            return Ok(());
        }

        // Takes care of non-compatible regions overlapping on the right of this region
        for other in map.entries() {
            if other.end() >= region.end_addr {
                if other.phys_start < region.end_addr {
                    if other.ty() > region.ty {
                        region.end_addr = other.phys_start;
                    } else {
                        region.end_addr = other.end();
                    }
                } else if other.phys_start == region.end_addr {
                    if other.ty() == region.ty {
                        region.end_addr = other.end();
                    }
                }
            }
        }

        if region.start_addr < region.end_addr {
            self.add_region(region)?;
        }

        Ok(())
    }

    fn entries(&self) -> core::slice::Iter<MemoryRegion> {
        self.as_slice().iter()
    }

    fn as_slice(&self) -> &[MemoryRegion] {
        unsafe {
            core::slice::from_raw_parts(
                self.regions.as_ptr() as *const MemoryRegion,
                self.num_regions,
            )
        }
    }

    fn as_mut_slice(&mut self) -> &mut [MemoryRegion] {
        unsafe {
            core::slice::from_raw_parts_mut(
                self.regions.as_mut_ptr() as *mut MemoryRegion,
                self.num_regions,
            )
        }
    }
}

trait MemoryArea {
    fn end(&self) -> u64;
    fn ty(&self) -> MemoryRegionType;
}

impl MemoryArea for MemoryDescriptor {
    fn end(&self) -> u64 {
        self.phys_start + self.page_count * 4096
    }

    fn ty(&self) -> MemoryRegionType {
        match self.ty {
            MemoryType::CONVENTIONAL
            | MemoryType::BOOT_SERVICES_CODE
            | MemoryType::BOOT_SERVICES_DATA => MemoryRegionType::Usable,
            MemoryType::LOADER_DATA | MemoryType::LOADER_CODE => {
                MemoryRegionType::BootloaderAllocated
            }
            MemoryType::ACPI_RECLAIM => MemoryRegionType::ACPIReclaimable,
            _ => MemoryRegionType::Reserved,
        }
    }
}

impl Drop for StackMemoryRegions {
    fn drop(&mut self) {
        unsafe {
            core::ptr::drop_in_place(self.as_mut_slice());
        }
    }
}

struct MemoryRegions {
    start: *mut MemoryRegion,
    num_regions: usize,
}

impl MemoryRegions {
    fn new(buffer: *mut MemoryRegion) -> Self {
        Self {
            start: buffer,
            num_regions: 0,
        }
    }

    fn add_region(&mut self, region: MemoryRegion) -> Result<(), GetMemoryMapError> {
        if self.num_regions >= MAX_NUM_REGIONS {
            return Err(GetMemoryMapError::TooManyMemoryRegionsErrror);
        }

        let next_ptr = unsafe { self.start.add(self.num_regions) };

        unsafe {
            next_ptr.write(region);
        }

        self.num_regions += 1;

        Ok(())
    }

    fn extend_from(
        &mut self,
        iter: core::slice::Iter<MemoryRegion>,
    ) -> Result<(), GetMemoryMapError> {
        for region in iter {
            self.add_region(*region)?;
        }

        Ok(())
    }

    fn entries(&self) -> core::slice::Iter<MemoryRegion> {
        self.as_slice().iter()
    }

    fn as_slice(&self) -> &[MemoryRegion] {
        unsafe { core::slice::from_raw_parts(self.start as *const MemoryRegion, self.num_regions) }
    }

    fn into_parts(self) -> (*const MemoryRegion, u64) {
        (self.start as *const MemoryRegion, self.num_regions as u64)
    }
}
