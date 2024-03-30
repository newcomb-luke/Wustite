use common::{ALL_PAGE_TABLES_END_ADDR, PAGE_MAP_LEVEL_4_TABLE_START_ADDR, PAGE_SIZE};

use crate::{bios::bios_get_next_segment, INITRAMFS_LOAD_LOCATION, INITRAMFS_LOAD_LOCATION_SIZE};

const MEMORY_REGIONS_DESCRIPTOR_ADDR: *mut u8 = 0x1000 as *mut u8;
const MEMORY_REGIONS_START_ADDR: *mut u64 = 0x1010 as *mut u64;

const SMAP_ENTRIES_START: *mut SMAPEntry = 0x00010000 as *mut SMAPEntry;

const KERNEL_EXECUTE_LOCATION: u64 = 0x00100000;
const KERNEL_EXECUTE_SIZE: u64 = 0x00200000;
const KERNEL_STACK_LOCATION: u64 = 0x00300000;
const KERNEL_STACK_SIZE: u64 = 0x00100000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    pub _acpi: u32,
}

impl SMAPEntry {
    pub fn end(&self) -> u64 {
        (self.base + self.length) - 1
    }
}

#[derive(Clone, Copy)]
struct MemoryRegion {
    pub start: u64,
    pub end: u64,
}

#[derive(Clone, Copy)]
struct MemoryRegionsDescriptor {
    num_regions: usize,
}

impl MemoryRegionsDescriptor {
    fn new() -> Self {
        unsafe { MEMORY_REGIONS_DESCRIPTOR_ADDR.write(0) }

        Self { num_regions: 0 }
    }

    fn inc_num_regions(&mut self) -> Result<(), MemoryDetectionError> {
        if self.num_regions >= u8::MAX as usize {
            Err(MemoryDetectionError::TooManyRegionsError)
        } else {
            self.num_regions += 1;
            unsafe { MEMORY_REGIONS_DESCRIPTOR_ADDR.write(self.num_regions as u8) }
            Ok(())
        }
    }

    fn get_max_usable_addr(&self) -> u64 {
        self.into_iter().last().map(|r| r.end).unwrap_or(0)
    }

    fn get_region(&self, index: usize) -> Option<MemoryRegion> {
        if index >= self.num_regions {
            return None;
        }

        let start_addr = unsafe { MEMORY_REGIONS_START_ADDR.add(index * 2) };

        Some(MemoryRegion {
            start: unsafe { start_addr.read_volatile() },
            end: unsafe { start_addr.add(1).read_volatile() },
        })
    }

    fn add_unified(&mut self, region: MemoryRegion) -> Result<(), MemoryDetectionError> {
        let start_addr = unsafe { MEMORY_REGIONS_START_ADDR.add(self.num_regions * 2) };

        unsafe {
            start_addr.write_volatile(region.start);
            start_addr.add(1).write_volatile(region.end);
        }

        self.inc_num_regions()
    }

    fn page_align(region: MemoryRegion) -> MemoryRegion {
        const PAGE_SIZE_U64: u64 = PAGE_SIZE as u64;

        let mut start = (region.start / PAGE_SIZE_U64) * PAGE_SIZE_U64;

        if (region.start % PAGE_SIZE_U64) != 0 {
            start += PAGE_SIZE_U64;
        }

        let mut end = (region.end / PAGE_SIZE_U64) * PAGE_SIZE_U64;

        if (region.end % PAGE_SIZE_U64) != 0 {
            end -= PAGE_SIZE_U64;
        }

        MemoryRegion { start, end }
    }

    fn constrain_usable(
        &mut self,
        entries: &SMAPEntries,
        mut usable: MemoryRegion,
    ) -> Result<(), MemoryDetectionError> {
        if usable.start >= usable.end {
            return Ok(());
        }

        for other in entries.sorted() {
            if other.base <= usable.start
                && other.end() >= usable.start
                && other.entry_type != SMAPEntryType::Usable
            {
                usable.start = usable.start.max(other.end() + 1);
            }
        }

        if usable.start >= usable.end {
            return Ok(());
        }

        for other in entries.sorted() {
            if other.base <= usable.end
                && other.end() >= usable.end
                && other.entry_type != SMAPEntryType::Usable
            {
                usable.end = usable.end.min(other.base - 1);
            }
        }

        if usable.start >= usable.end {
            return Ok(());
        }

        for other in entries.sorted() {
            if other.base > usable.start
                && other.end() < usable.end
                && other.entry_type != SMAPEntryType::Usable
            {
                let first_region = MemoryRegion {
                    start: usable.start,
                    end: other.base - 1,
                };

                self.constrain_usable(entries, first_region)?;

                let second_region = MemoryRegion {
                    start: other.end(),
                    end: usable.end,
                };

                self.constrain_usable(entries, second_region)?;

                return Ok(());
            }
        }

        if usable.start < usable.end {
            let aligned = Self::page_align(usable);

            if aligned.start < aligned.end {
                self.add_unified(aligned)?;
            }
        }

        Ok(())
    }

    fn unify_regions(&mut self, smap_entries: SMAPEntries) -> Result<(), MemoryDetectionError> {
        for entry in smap_entries.sorted() {
            if entry.entry_type == SMAPEntryType::Usable {
                let usable = MemoryRegion {
                    start: entry.base,
                    end: entry.end(),
                };

                self.constrain_usable(&smap_entries, usable)?;
            }
        }

        Ok(())
    }

    fn num_regions(&self) -> usize {
        self.num_regions
    }
}

impl IntoIterator for MemoryRegionsDescriptor {
    type Item = MemoryRegion;
    type IntoIter = MemoryRegionsIterator;

    fn into_iter(self) -> Self::IntoIter {
        MemoryRegionsIterator {
            descriptor: self,
            index: 0,
        }
    }
}

struct MemoryRegionsIterator {
    descriptor: MemoryRegionsDescriptor,
    index: usize,
}

impl Iterator for MemoryRegionsIterator {
    type Item = MemoryRegion;

    fn next(&mut self) -> Option<Self::Item> {
        let item = self.descriptor.get_region(self.index)?;

        self.index += 1;

        Some(item)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (
            self.descriptor.num_regions(),
            Some(self.descriptor.num_regions()),
        )
    }
}

#[derive(Debug, Clone, Copy)]
pub enum MemoryDetectionError {
    TooManyRegionsError,
    BIOSError,
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

    #[inline(never)]
    fn next(&mut self) -> Result<Option<SMAPEntry>, MemoryDetectionError> {
        if !self.first && (self.bytes_read <= 0 || self.continuation_id == 0) {
            return Ok(None);
        }
        self.first = false;

        let mut buffer: [u8; 24] = [0; 24];

        self.bytes_read =
            unsafe { bios_get_next_segment(buffer.as_mut_ptr(), &mut self.continuation_id) };
        // unsafe { _BIOS_Memory_GetNextSegment(&mut buffer, &mut self.continuation_id) };

        // I really have no idea what the problem is, but we need to do this
        // in order for Rust to know to initially set the values to zero, otherwise
        // it just leaves whatever was in stack memory in there, and only copies
        // in the lower half of the u64's
        let mut output = SMAPEntry {
            base: 0,
            length: 0,
            entry_type: SMAPEntryType::Reserved,
            _acpi: 1,
        };

        let mut base_bytes: [u8; 8] = [0; 8];
        base_bytes.copy_from_slice(&buffer[0..8]);

        let mut length_bytes: [u8; 8] = [0; 8];
        length_bytes.copy_from_slice(&buffer[8..16]);

        let mut entry_type_bytes: [u8; 4] = [0; 4];
        entry_type_bytes.copy_from_slice(&buffer[16..20]);

        let mut acpi_bytes: [u8; 4] = [0; 4];
        acpi_bytes.copy_from_slice(&buffer[20..24]);

        let base = u64::from_ne_bytes(base_bytes);
        let length = u64::from_ne_bytes(length_bytes);
        let entry_type_raw = u32::from_ne_bytes(entry_type_bytes);
        let acpi = u32::from_ne_bytes(acpi_bytes);

        output.base = base;
        output.length = length;
        output.entry_type = entry_type_raw.into();
        output._acpi = acpi;

        if self.bytes_read < 0 {
            Err(MemoryDetectionError::BIOSError)
        } else {
            Ok(Some(output))
        }
    }
}

#[derive(Clone, Copy)]
struct SMAPEntries {
    num_entries: usize,
}

impl SMAPEntries {
    fn add_entry(&mut self, entry: SMAPEntry) -> Result<(), MemoryDetectionError> {
        if self.num_entries >= u8::MAX as usize {
            return Err(MemoryDetectionError::TooManyRegionsError);
        }

        let next_entry_addr = unsafe { SMAP_ENTRIES_START.add(self.num_entries) };
        unsafe { next_entry_addr.write_volatile(entry) };

        self.num_entries += 1;

        Ok(())
    }

    // This function will succeed as long as the place where the SMAPEntry's are being stored.
    // References must be aligned, and on 32 and 64 bit, SMAPEntry's will be aligned in memory.
    fn get_entry(&self, index: usize) -> Option<&'static SMAPEntry> {
        if index >= self.num_entries() {
            return None;
        }

        let entry_addr = unsafe { SMAP_ENTRIES_START.add(index) };

        unsafe { entry_addr.as_ref() }
    }

    fn read_from_bios() -> Result<Self, MemoryDetectionError> {
        let mut entries = Self { num_entries: 0 };

        let mut entries_reader = SMAPEntriesReader::new();

        while let Some(entry) = entries_reader.next()? {
            entries.add_entry(entry)?;
        }

        Ok(entries)
    }

    fn max_base_entry(&self) -> Option<SMAPEntry> {
        self.into_iter().max_by_key(|e| e.base)
    }

    fn max_base_entry_index(&self) -> Option<usize> {
        self.into_iter()
            .enumerate()
            .max_by_key(|(_, e)| e.base)
            .map(|(i, _)| i)
    }

    fn sorted(&self) -> SortedSMAPEntriesIterator {
        SortedSMAPEntriesIterator {
            entries: *self,
            visited: [false; 256],
            addr: 0,
        }
    }

    fn num_entries(&self) -> usize {
        self.num_entries
    }
}

impl IntoIterator for SMAPEntries {
    type Item = SMAPEntry;
    type IntoIter = SMAPEntriesIterator;

    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter {
            entries: self,
            next_entry: 0,
        }
    }
}

struct SortedSMAPEntriesIterator {
    entries: SMAPEntries,
    visited: [bool; 256],
    addr: u64,
}

impl Iterator for SortedSMAPEntriesIterator {
    type Item = SMAPEntry;

    fn next(&mut self) -> Option<Self::Item> {
        let mut min_entry = self.entries.max_base_entry()?;
        let mut min_entry_index = self.entries.max_base_entry_index()?;
        let mut min_found = false;

        if min_entry.base < self.addr {
            return None;
        }

        for (i, entry) in self.entries.into_iter().enumerate() {
            if !self.visited[i] && entry.base >= self.addr && entry.base <= min_entry.base {
                min_entry = entry;
                min_entry_index = i;
                min_found = true;
            }
        }

        if !min_found {
            self.addr = min_entry.base + 1;
            return self.next();
        }

        self.visited[min_entry_index] = true;

        Some(min_entry)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.entries.num_entries(), Some(self.entries.num_entries()))
    }
}

struct SMAPEntriesIterator {
    entries: SMAPEntries,
    next_entry: usize,
}

impl Iterator for SMAPEntriesIterator {
    type Item = SMAPEntry;

    fn next(&mut self) -> Option<Self::Item> {
        let entry = self.entries.get_entry(self.next_entry)?;
        self.next_entry += 1;

        Some(*entry)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.entries.num_entries(), Some(self.entries.num_entries()))
    }
}

pub fn detect_memory_regions() -> Result<u64, MemoryDetectionError> {
    // Initialize the global memory regions descriptor
    let mut memory_regions_descriptor = MemoryRegionsDescriptor::new();

    // Read the SMAP entries from the BIOS
    let mut smap_entries = SMAPEntries::read_from_bios()?;

    // Add where the kernel is loaded, that would be bad if it wasn't included
    smap_entries.add_entry(SMAPEntry {
        base: KERNEL_EXECUTE_LOCATION,
        length: KERNEL_EXECUTE_SIZE,
        entry_type: SMAPEntryType::Reserved,
        _acpi: 1,
    })?;

    // Also add the kernel stack, this could theoretically be separate, so we add
    // it separately
    smap_entries.add_entry(SMAPEntry {
        base: KERNEL_STACK_LOCATION,
        length: KERNEL_STACK_SIZE,
        entry_type: SMAPEntryType::Reserved,
        _acpi: 1,
    })?;

    // Add where the page tables are created
    smap_entries.add_entry(SMAPEntry {
        base: PAGE_MAP_LEVEL_4_TABLE_START_ADDR,
        length: ALL_PAGE_TABLES_END_ADDR - PAGE_MAP_LEVEL_4_TABLE_START_ADDR,
        entry_type: SMAPEntryType::Reserved,
        _acpi: 1,
    })?;

    // Add where the initramfs is loaded
    smap_entries.add_entry(SMAPEntry {
        base: INITRAMFS_LOAD_LOCATION as u64,
        length: INITRAMFS_LOAD_LOCATION_SIZE as u64,
        entry_type: SMAPEntryType::Reserved,
        _acpi: 1,
    })?;

    memory_regions_descriptor.unify_regions(smap_entries)?;

    Ok(memory_regions_descriptor.get_max_usable_addr())
}
