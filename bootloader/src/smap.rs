use core::fmt::Display;

use crate::{
    paging::{PAGE_MAP_LEVEL_4_TABLE_START, PAGE_TABLES_LENGTH},
    println,
};

const MEMORY_REGIONS_DESCRIPTOR_ADDR: *mut MemoryRegionsDescriptor =
    0x500 as *mut MemoryRegionsDescriptor;
const MEMORY_REGIONS_START_ADDR: *mut MemoryRegion = 0x600 as *mut MemoryRegion;

const SMAP_ENTRIES_START: *mut SMAPEntry = 0x00010000 as *mut SMAPEntry;

const KERNEL_EXECUTE_LOCATION: u64 = 0x00100000;
const KERNEL_EXECUTE_SIZE: u64 = 0x001FFFFF;
const KERNEL_STACK_LOCATION: u64 = 0x00300000;
const KERNEL_STACK_SIZE: u64 = 0x00100000;

#[link(name = "bios")]
extern "cdecl" {
    fn _BIOS_Memory_GetNextSegment(entry: *mut [u8; 24], continuation_id: *mut u32) -> i32;
}

#[derive(Debug, Clone, Copy)]
enum SMAPEntryType {
    Usable = 1,
    Reserved = 2,
    ACPIReclaimable = 3,
    ACPINVS = 4,
    BadMemory = 5,
    Unknown,
}

impl From<u32> for SMAPEntryType {
    fn from(value: u32) -> Self {
        match value {
            1 => SMAPEntryType::Usable,
            2 => SMAPEntryType::Reserved,
            3 => SMAPEntryType::ACPIReclaimable,
            4 => SMAPEntryType::ACPINVS,
            5 => SMAPEntryType::BadMemory,
            _ => SMAPEntryType::Unknown,
        }
    }
}

#[derive(Clone, Copy)]
struct SMAPEntry {
    pub base: u64,
    pub length: u64,
    pub entry_type: SMAPEntryType,
    pub acpi: u32,
}

impl Display for SMAPEntry {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!(
            "Entry base: 0x{:016x}, size: 0x{:016x}, type: {:?}",
            self.base, self.length, self.entry_type
        ))
    }
}

impl From<[u8; 24]> for SMAPEntry {
    fn from(value: [u8; 24]) -> Self {
        let mut base_bytes: [u8; 8] = [0; 8];
        base_bytes.copy_from_slice(&value[0..8]);
        let mut length_bytes: [u8; 8] = [0; 8];
        length_bytes.copy_from_slice(&value[8..16]);
        let mut entry_type_bytes: [u8; 4] = [0; 4];
        entry_type_bytes.copy_from_slice(&value[16..20]);
        let mut acpi_bytes: [u8; 4] = [0; 4];
        acpi_bytes.copy_from_slice(&value[20..24]);

        let base = u64::from_ne_bytes(base_bytes);
        let length = u64::from_ne_bytes(length_bytes);
        let entry_type_raw = u32::from_ne_bytes(entry_type_bytes);
        let acpi = u32::from_ne_bytes(acpi_bytes);

        SMAPEntry {
            base,
            length,
            entry_type: entry_type_raw.into(),
            acpi,
        }
    }
}

#[derive(Clone, Copy)]
#[repr(u8)]
enum MemoryRegionType {
    Usable = 1,
    Reserved = 2,
    ACPIReclaimable = 3,
}

// These are repr(C) so that they are *guaranteed* to be ABI compatible with the kernel
// when it goes to read it
#[repr(C)]
struct MemoryRegion {
    start: u64,
    end: u64,
    region_type: MemoryRegionType,
}

#[repr(C)]
struct MemoryRegionsDescriptor {
    num_regions: u8,
    start: *mut MemoryRegion,
}

#[derive(Debug, Clone, Copy)]
pub enum MemoryDetectionError {
    TooManyRegionsError,
    BIOSError,
}

impl MemoryRegionsDescriptor {
    fn add_region(&mut self, region: MemoryRegion) -> Result<(), MemoryDetectionError> {
        if self.num_regions >= u8::MAX {
            return Err(MemoryDetectionError::TooManyRegionsError);
        }

        unsafe {
            let next_region = self.start.offset(self.num_regions as isize);

            next_region.write(region);

            self.num_regions += 1;
        }

        Ok(())
    }
}

struct SMAPEntriesReader {
    first: bool,
    bytes_read: i32,
    continuation_id: u32,
}

impl SMAPEntriesReader {
    fn new() -> Self {
        Self {
            first: true,
            bytes_read: 0,
            continuation_id: 0,
        }
    }
}

impl Iterator for SMAPEntriesReader {
    type Item = SMAPEntry;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.first && (self.bytes_read <= 0 || self.continuation_id == 0) {
            return None;
        }
        self.first = false;

        let mut buffer: [u8; 24] = [0; 24];

        self.bytes_read =
            unsafe { _BIOS_Memory_GetNextSegment(&mut buffer, &mut self.continuation_id) };

        // Signals an error
        if self.bytes_read < 0 {
            None
        } else {
            Some(buffer.into())
        }
    }
}

#[derive(Clone, Copy)]
struct SMAPEntries {
    num_entries: usize,
}

impl SMAPEntries {
    // fn add_entry(&mut self, entry: SMAPEntry) -> Result<(), MemoryDetectionError> {
    //     if self.num_entries >= u8::MAX as usize {
    //         return Err(MemoryDetectionError::TooManyRegionsError);
    //     }

    //     let next_entry_addr = if self.num_entries() == 0 {
    //         SMAP_ENTRIES_START
    //     } else {
    //         unsafe { SMAP_ENTRIES_START.offset(self.num_entries as isize) }
    //     };
    //     unsafe { next_entry_addr.write(entry) };
    //     self.num_entries += 1;

    //     Ok(())
    // }

    /// This function will succeed as long as the place where the SMAPEntry's are being stored.
    /// References must be aligned, and on 32 and 64 bit, SMAPEntry's will be aligned in memory.
    // unsafe fn get_entry(&self, index: usize) -> Option<&'static SMAPEntry> {
    //     if index >= self.num_entries() {
    //         return None;
    //     }

    //     let entry_addr = unsafe { SMAP_ENTRIES_START.offset(index as isize) };

    //     unsafe { entry_addr.as_ref() }
    // }

    fn read_from_bios() -> Result<Self, MemoryDetectionError> {
        let mut entries = Self { num_entries: 0 };

        let mut entries_reader = SMAPEntriesReader::new();

        for mut entry in entries_reader {
            println!("{entry}");
        }

        Ok(entries)
    }

    fn num_entries(&self) -> usize {
        self.num_entries
    }
}

impl IntoIterator for SMAPEntries {
    type Item = &'static SMAPEntry;
    type IntoIter = SMAPEntriesIterator;

    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter {
            entries: self,
            next_entry: 0,
        }
    }
}

struct SMAPEntriesIterator {
    entries: SMAPEntries,
    next_entry: usize,
}

impl Iterator for SMAPEntriesIterator {
    type Item = &'static SMAPEntry;

    fn next(&mut self) -> Option<Self::Item> {
        // let entry = unsafe { self.entries.get_entry(self.next_entry) }?;
        let entry = SMAPEntry {
            base: 0xf,
            length: 0x20,
            entry_type: SMAPEntryType::Reserved,
            acpi: 0,
        };

        self.next_entry += 1;

        todo!();
    }
}

pub fn detect_memory_regions() -> Result<(), MemoryDetectionError> {
    // Initialize the global memory regions descriptor
    // let memory_regions_descriptor = unsafe {
    //     MEMORY_REGIONS_DESCRIPTOR_ADDR.write(MemoryRegionsDescriptor {
    //         num_regions: 0,
    //         start: MEMORY_REGIONS_START_ADDR,
    //     });

    //     MEMORY_REGIONS_DESCRIPTOR_ADDR.as_mut().unwrap()
    // };

    // Read the SMAP entries from the BIOS
    let mut smap_entries = SMAPEntries::read_from_bios()?;

    /*
    // Add where the kernel is loaded, that would be bad if it wasn't included
    smap_entries.add_entry(SMAPEntry {
        base: KERNEL_EXECUTE_LOCATION,
        length: KERNEL_EXECUTE_SIZE,
        entry_type: 2,
        acpi: 0,
    })?;

    // Also add the kernel stack, this could theoretically be separate, so we add
    // it separately
    smap_entries.add_entry(SMAPEntry {
        base: KERNEL_STACK_LOCATION,
        length: KERNEL_STACK_SIZE,
        entry_type: 2,
        acpi: 0,
    })?;

    // Add where the page tables are created
    smap_entries.add_entry(SMAPEntry {
        base: PAGE_MAP_LEVEL_4_TABLE_START as u64,
        length: PAGE_TABLES_LENGTH,
        entry_type: 2,
        acpi: 0,
    })?;
    */

    // for entry in smap_entries {
    //     println!("{}", entry);
    // }

    todo!();
}
