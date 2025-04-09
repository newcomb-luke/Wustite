use core::str;

use bin_tools::read_u32_le;
use block_device::BlockDevice;
use vfat32_core::{
    entry::{DirectoryEntry, RealEntry, DIRECTORY_ENTRY_SIZE},
    fs_info::FSInfo,
    record::BootRecord,
};

pub type DriverResult<T> = Result<T, DriverError>;

#[derive(Debug, Clone, Copy)]
pub enum DriverError {
    DiskError,
    FileSystemInvalid,
    PathNotFound,
    IsADirectory,
    IsNotADirectory,
}

struct EntryName {
    inner: [u8; 256],
    len: u8,
}

impl EntryName {
    fn new(inner: [u8; 256]) -> Self {
        let mut len = 0;

        for b in inner {
            if b != 0 {
                len += 1;
            } else {
                break;
            }
        }

        Self { inner, len }
    }

    fn str(&self) -> &str {
        unsafe { str::from_utf8_unchecked(&self.inner[0..(self.len as usize)]) }
    }
}

struct NamedEntry {
    name: EntryName,
    entry: RealEntry,
}

impl NamedEntry {
    fn new(name: [u8; 256], entry: RealEntry) -> Self {
        Self {
            name: EntryName::new(name),
            entry,
        }
    }

    fn name(&self) -> &str {
        self.name.str()
    }
}

struct DataBuffer {
    inner: [u8; 512],
    location: Option<u64>,
}

impl DataBuffer {
    fn empty() -> Self {
        Self {
            inner: [0u8; 512],
            location: None,
        }
    }

    fn as_slice(&self) -> &[u8] {
        &self.inner
    }

    fn as_slice_mut(&mut self) -> &mut [u8] {
        &mut self.inner
    }
}

pub struct VFATDirectoryEntry {
    named_entry: NamedEntry,
}

impl VFATDirectoryEntry {
    pub fn name(&self) -> &str {
        self.named_entry.name()
    }

    pub fn is_file(&self) -> bool {
        self.named_entry.entry.is_file()
    }

    pub fn is_directory(&self) -> bool {
        self.named_entry.entry.is_dir()
    }
}

pub struct VFATFile {
    start_cluster: u32,
    size_bytes: u32,
}

impl VFATFile {
    pub fn size(&self) -> usize {
        self.size_bytes as usize
    }
}

pub struct VFATDirectory {
    start_cluster: u32,
}

pub struct VFAT32Driver<D>
where
    D: BlockDevice,
{
    block_device: D,
    sector_size: u32,
    boot_record: BootRecord,
    fs_info: FSInfo,
    fat_buffer: DataBuffer,
    data_buffer: DataBuffer,
}

impl<D: BlockDevice> VFAT32Driver<D> {
    pub fn new(mut block_device: D) -> Result<Self, DriverError> {
        let mut buffer = [0u8; 512];
        block_device
            .read_block(0, &mut buffer)
            .map_err(|_| DriverError::DiskError)?;

        let boot_record = BootRecord::read(&buffer);

        let fs_info_sector = boot_record.fs_info_sector() as u64;
        block_device
            .read_block(fs_info_sector, &mut buffer)
            .map_err(|_| DriverError::DiskError)?;

        let fs_info = FSInfo::read(&buffer);

        if !fs_info.is_valid() {
            return Err(DriverError::FileSystemInvalid);
        }

        Ok(Self {
            block_device,
            sector_size: boot_record.bytes_per_sector() as u32,
            boot_record,
            fs_info,
            fat_buffer: DataBuffer::empty(),
            data_buffer: DataBuffer::empty(),
        })
    }

    pub fn open_dir(&mut self, path: &str) -> DriverResult<VFATDirectory> {
        if Self::iter_path_segments(path).next().is_none() {
            // This is empty of real path segments, but maybe not completely empty
            if !path.is_empty() {
                // This is the root directory
                return Ok(VFATDirectory {
                    start_cluster: self.boot_record.root_directory_cluster(),
                });
            } else {
                // This is really empty, which is not a real path
                return Err(DriverError::PathNotFound);
            }
        }

        let found_entry = self.open_entry(path)?;

        if !found_entry.entry.is_dir() {
            return Err(DriverError::IsNotADirectory)?;
        }

        Ok(VFATDirectory {
            start_cluster: found_entry.entry.start_cluster(),
        })
    }

    pub fn read_dir(
        &mut self,
        directory: VFATDirectory,
    ) -> DriverResult<impl Iterator<Item = DriverResult<VFATDirectoryEntry>> + use<'_, D>> {
        Ok(self
            .iter_directory_entries(directory.start_cluster)
            .map(|e| e.map(|e| VFATDirectoryEntry { named_entry: e })))
    }

    pub fn open(&mut self, path: &str) -> DriverResult<VFATFile> {
        let found_entry = self.open_entry(path)?;

        if !found_entry.entry.is_file() {
            return Err(DriverError::IsADirectory)?;
        }

        Ok(VFATFile {
            start_cluster: found_entry.entry.start_cluster(),
            size_bytes: found_entry.entry.file_size(),
        })
    }

    pub fn read_file(
        &mut self,
        file: VFATFile,
        offset: usize,
        buffer: &mut [u8],
    ) -> DriverResult<usize> {
        let start_sector_index = offset / self.boot_record.bytes_per_sector() as usize;
        let bytes_per_sector = self.boot_record.bytes_per_sector() as usize;

        let mut current_file_offset = offset;
        let mut current_sector = 0;
        let mut next_cluster = file.start_cluster;
        let mut bytes_read = 0;

        while !Self::is_end(next_cluster) {
            for sector_in_cluster in 0..self.boot_record.sectors_per_cluster() {
                self.read_data_cluster(next_cluster, sector_in_cluster)?;

                // This sector is special because we could have an offset in it
                if current_sector == start_sector_index {
                    // If the offset is 0, then this just starts at 0
                    let start = current_file_offset % bytes_per_sector;
                    let end = bytes_per_sector.min(buffer.len() - bytes_read);

                    for i in start..end {
                        buffer[bytes_read] = self.data_buffer.as_slice()[i];
                        bytes_read += 1;
                    }

                    current_file_offset = 0;
                } else {
                    let end = bytes_per_sector.min(buffer.len() - bytes_read);

                    for i in 0..end {
                        buffer[bytes_read] = self.data_buffer.as_slice()[i];
                        bytes_read += 1;
                    }
                }

                if bytes_read >= buffer.len() {
                    return Ok(bytes_read);
                }

                current_sector += 1;
            }

            next_cluster = self.read_fat_entry(next_cluster)?;
        }

        Ok(bytes_read)
    }

    fn open_entry<'a>(&'a mut self, path: &str) -> DriverResult<NamedEntry> {
        let mut segments = Self::iter_path_segments(path).peekable();

        let mut current_dir_cluster = self.boot_record.root_directory_cluster();

        while let Some(segment) = segments.next() {
            let dir_entry = self
                .find_dir_entry(segment, current_dir_cluster)?
                .ok_or(DriverError::PathNotFound)?;

            if segments.peek().is_none() {
                // This is the last segment of the path
                return Ok(dir_entry);
            } else {
                // This is a directory segment
                if !dir_entry.entry.is_dir() {
                    return Err(DriverError::PathNotFound)?;
                }

                current_dir_cluster = dir_entry.entry.start_cluster();
            }
        }

        Err(DriverError::PathNotFound)
    }

    fn iter_path_segments(path: &str) -> impl Iterator<Item = &str> {
        path.split('/').filter(|s| !s.is_empty())
    }

    fn find_dir_entry(
        &mut self,
        name: &str,
        dir_start_cluster: u32,
    ) -> Result<Option<NamedEntry>, DriverError> {
        for entry in self.iter_directory_entries(dir_start_cluster) {
            let entry = entry?;

            if entry.name() == name {
                return Ok(Some(entry));
            }
        }

        return Ok(None);
    }

    fn iter_directory_entries(
        &mut self,
        dir_start_cluster: u32,
    ) -> impl Iterator<Item = DriverResult<NamedEntry>> + use<'_, D> {
        NamedEntryIterator::new(self, dir_start_cluster)
    }

    fn read_dir_entry_from_buffer(&self, index: usize) -> Option<DirectoryEntry> {
        let byte_index = index * DIRECTORY_ENTRY_SIZE;
        let entry_slice = &self.data_buffer.as_slice()[byte_index..];
        let entry = DirectoryEntry::read(entry_slice);

        match entry {
            DirectoryEntry::Real(real) => {
                if real.is_empty() {
                    None
                } else {
                    Some(entry)
                }
            }
            lfn => Some(lfn),
        }
    }

    fn read_fat_sector(&mut self, cluster: u32) -> Result<usize, DriverError> {
        let (sector, offset) = self.sector_in_fat(cluster);

        if let Some(buffered_sector) = self.fat_buffer.location {
            if sector == buffered_sector {
                return Ok(offset);
            }
        }

        self.fat_buffer.location = Some(sector);
        self.block_device
            .read_block(sector, self.fat_buffer.as_slice_mut())
            .map_err(|_| DriverError::DiskError)?;

        Ok(offset)
    }

    fn read_dir_entry(
        &mut self,
        start_cluster: u32,
        entry_index: usize,
    ) -> Result<Option<DirectoryEntry>, DriverError> {
        let sector_index =
            (entry_index * DIRECTORY_ENTRY_SIZE) / self.boot_record.bytes_per_sector() as usize;
        let entries_per_sector =
            self.boot_record.bytes_per_sector() as usize / DIRECTORY_ENTRY_SIZE;

        let mut current_sector = 0;
        let mut next_cluster = start_cluster;

        while !Self::is_end(next_cluster) {
            for sector_in_cluster in 0..self.boot_record.sectors_per_cluster() {
                self.read_data_cluster(next_cluster, sector_in_cluster)?;

                if current_sector == sector_index {
                    let index_into_buffer = entry_index % entries_per_sector;

                    return Ok(self.read_dir_entry_from_buffer(index_into_buffer));
                }

                current_sector += 1;
            }

            next_cluster = self.read_fat_entry(next_cluster)?;
        }

        Ok(None)
    }

    fn is_end(cluster: u32) -> bool {
        cluster >= 0x0FFFFFF8
    }

    fn read_fat_entry(&mut self, cluster: u32) -> Result<u32, DriverError> {
        let fat_entry_offset = self.read_fat_sector(cluster)?;

        let data_slice = self.fat_buffer.as_slice();
        let entry = read_u32_le(data_slice, fat_entry_offset);

        Ok(entry & 0x0FFFFFFF)
    }

    fn read_data_cluster(&mut self, cluster: u32, sector_offset: u64) -> Result<(), DriverError> {
        let sector = self.data_sector_from_cluster(cluster);

        self.read_data_sector(sector + sector_offset)?;

        Ok(())
    }

    fn read_data_sector(&mut self, sector: u64) -> Result<(), DriverError> {
        if let Some(buffered_sector) = self.data_buffer.location {
            if sector == buffered_sector {
                return Ok(());
            }
        }

        self.data_buffer.location = Some(sector);
        self.block_device
            .read_block(sector, self.data_buffer.as_slice_mut())
            .map_err(|_| DriverError::DiskError)
    }

    fn cluster_to_relative_sector(&self, cluster: u32) -> u64 {
        ((cluster - 2) * self.boot_record.sectors_per_cluster() as u32) as u64
    }

    fn data_sector_from_cluster(&self, cluster: u32) -> u64 {
        self.cluster_to_relative_sector(cluster) + self.boot_record.first_data_sector()
    }

    fn sector_in_fat(&self, cluster: u32) -> (u64, usize) {
        let fat_start_sector = self.boot_record.first_fat_sector() as u64;
        let fat_offset = cluster * 4; // 4 bytes per 32-bit entry

        let sector = fat_start_sector + (fat_offset as u64 / self.sector_size as u64);
        let offset_into_sector = fat_offset % (self.sector_size);

        (sector, offset_into_sector as usize)
    }
}

struct NamedEntryIterator<'a, D>
where
    D: BlockDevice,
{
    driver: &'a mut VFAT32Driver<D>,
    start_cluster: u32,
    next_index: usize,
}

impl<'a, D> NamedEntryIterator<'a, D>
where
    D: BlockDevice,
{
    fn new(driver: &'a mut VFAT32Driver<D>, start_cluster: u32) -> Self {
        Self {
            driver,
            start_cluster,
            next_index: 0,
        }
    }
}

impl<'a, D> Iterator for NamedEntryIterator<'a, D>
where
    D: BlockDevice,
{
    type Item = DriverResult<NamedEntry>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut current_file_name_index = 0;
        let mut current_file_name = [0u8; 256];
        let mut lfn_first = None;

        loop {
            if let Ok(entry) = self
                .driver
                .read_dir_entry(self.start_cluster, self.next_index)
                .transpose()?
            {
                match entry {
                    DirectoryEntry::LFN(_) => {
                        if lfn_first.is_none() {
                            lfn_first = Some(self.next_index);
                        }
                        self.next_index += 1;
                    }
                    DirectoryEntry::Real(entry) => {
                        let current_index = self.next_index;
                        self.next_index += 1;

                        if let Some(first_idx) = lfn_first {
                            for lfn_entry_idx in (first_idx..current_index).rev() {
                                if let Ok(DirectoryEntry::LFN(lfn_entry)) = self
                                    .driver
                                    .read_dir_entry(self.start_cluster, lfn_entry_idx)
                                    .transpose()?
                                {
                                    for c in lfn_entry.name() {
                                        current_file_name[current_file_name_index] = c as u8;
                                        current_file_name_index += 1;
                                    }
                                } else {
                                    panic!();
                                }
                            }

                            return Some(Ok(NamedEntry::new(current_file_name, entry)));
                        } else {
                            let name_bytes = entry.name_bytes();

                            // Get the file name
                            for i in 0..8 {
                                let c = name_bytes[i];

                                if c != b' ' {
                                    if entry.is_name_lowercase() {
                                        current_file_name[current_file_name_index] =
                                            c.to_ascii_lowercase();
                                    } else {
                                        current_file_name[current_file_name_index] = c;
                                    }
                                    current_file_name_index += 1;
                                }
                            }
                            if entry.has_extension() {
                                // If the file name has an extension, add a dot
                                current_file_name[current_file_name_index] = b'.';
                                current_file_name_index += 1;

                                // Get the extension
                                for i in 8..11 {
                                    let c = name_bytes[i];

                                    if c != b' ' {
                                        if entry.is_extension_lowercase() {
                                            current_file_name[current_file_name_index] =
                                                c.to_ascii_lowercase();
                                        } else {
                                            current_file_name[current_file_name_index] = c;
                                        }
                                        current_file_name_index += 1;
                                    }
                                }
                            }

                            return Some(Ok(NamedEntry::new(current_file_name, entry)));
                        }
                    }
                }
            }
        }
    }
}
