#![allow(dead_code)]

use core::mem::size_of;

use crate::{
    disk::{Disk, DiskReadError, SECTOR_SIZE},
    println,
};

const FAT_DRIVER_BOOT_SECTOR_PTR: *mut u8 = 0x7c00 as *mut u8;

const FAT_DRIVER_ROOT_DIR_BUFFER_PTR: *mut u8 = 0x8000 as *mut u8;
const FAT_DRIVER_ROOT_DIR_BUFFER_SECTORS: u16 = 2;

const FAT_DRIVER_FAT_BUFFER_PTR: *mut u8 = 0x8400 as *mut u8;
const FAT_DRIVER_FAT_BUFFER_SECTORS: u16 = 3;

const FAT_DRIVER_FILE_BUFFER_PTR: *mut u8 = 0x9000 as *mut u8;
const FAT_DRIVER_FILE_BUFFER_SECTORS: u16 = 2;

#[derive(Clone, Copy)]
#[repr(u8)]
enum EntryAttribute {
    ReadOnly = 0x01,
    Hidden = 0x02,
    System = 0x04,
    VolumeId = 0x08,
    Directory = 0x10,
    Archive = 0x20,
    LargeFileName =
        Self::ReadOnly as u8 | Self::Hidden as u8 | Self::System as u8 | Self::VolumeId as u8,
}

#[derive(Debug, Clone, Copy)]
pub enum FATDriverError {
    DiskReadError,
    FileNotFoundError,
    UnsupportedSectorsPerClusterError,
    InvalidFileSizeError,
}

#[derive(Clone, Copy)]
struct FATLabel([u8; 11]);

impl FATLabel {
    pub fn as_str(&self) -> &str {
        unsafe { core::str::from_utf8_unchecked(&self.0) }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum FileNameError {
    NameTooLongError,
    ExtensionTooLongError,
}

#[derive(Clone, Copy)]
#[repr(C, packed)]
pub struct FileName([u8; 11]);

impl FileName {
    pub fn as_str(&self) -> &str {
        unsafe { core::str::from_utf8_unchecked(&self.0) }
    }

    pub fn name(&self) -> &str {
        unsafe { core::str::from_utf8_unchecked(&self.0[0..8]) }
    }

    pub fn extension(&self) -> &str {
        unsafe { core::str::from_utf8_unchecked(&self.0[8..]) }
    }
}

impl TryFrom<&str> for FileName {
    type Error = FileNameError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut buf = [' ' as u8; 11];

        let mut got_name = false;
        let mut index = 0;

        for c in value.bytes() {
            if c == '.' as u8 {
                index = 0;
                got_name = true;
                continue;
            }

            let c_uppercase = if c < 97 { c } else { c - 32 };

            if !got_name {
                let name_buf = &mut buf[0..8];

                if index >= name_buf.len() {
                    return Err(FileNameError::NameTooLongError);
                }

                name_buf[index] = c_uppercase;
            } else {
                let ext_buf = &mut buf[8..];

                if index >= ext_buf.len() {
                    return Err(FileNameError::ExtensionTooLongError);
                }

                ext_buf[index] = c_uppercase;
            }

            index += 1;
        }

        Ok(FileName(buf))
    }
}

impl PartialEq for FileName {
    fn eq(&self, other: &Self) -> bool {
        for i in 0..11 {
            if self.0[i] != other.0[i] {
                return false;
            }
        }

        true
    }
}

#[derive(Clone, Copy)]
#[repr(C, packed)]
struct DirEntryDate {
    date: u16,
}

#[derive(Clone, Copy)]
#[repr(C, packed)]
struct FileCreationTime {
    time_tenths: u8,
    time: u16,
    date: DirEntryDate,
}

#[derive(Clone, Copy)]
#[repr(C, packed)]
struct FileModificationTime {
    time: u16,
    date: DirEntryDate,
}

#[derive(Clone, Copy)]
#[repr(C, packed)]
struct DirEntry {
    name: FileName,
    attributes: u8,
    __reserved: u8,
    creation_time: FileCreationTime,
    last_accessed_date: DirEntryDate,
    first_cluster_high: u16,
    last_modified_time: FileModificationTime,
    first_cluster_low: u16,
    file_size: u32,
}

impl DirEntry {
    fn is_file(&self) -> bool {
        (self.attributes & EntryAttribute::Directory as u8) == 0
    }

    fn is_end(&self) -> bool {
        self.name.0[0] == 0
    }

    fn is_erased(&self) -> bool {
        self.name.0[0] == 0x05 || self.name.0[0] == 0xe5
    }

    fn is_dot(&self) -> bool {
        self.name.0[0] == 0x2e
    }

    fn is_volume_id(&self) -> bool {
        (self.attributes & EntryAttribute::VolumeId as u8) != 0
    }

    fn first_cluster_low(&self) -> u16 {
        unsafe { core::ptr::addr_of!(self.first_cluster_low).read_unaligned() }
    }

    fn file_size(&self) -> u32 {
        unsafe { core::ptr::addr_of!(self.file_size).read_unaligned() }
    }
}

#[derive(Clone, Copy)]
struct BootRecordOEM([u8; 8]);

#[derive(Clone, Copy)]
struct BootRecordVolumeId([u8; 4]);

#[derive(Clone, Copy)]
struct BootRecordSystemId([u8; 8]);

/// Only the useful parts of the boot record
#[derive(Clone, Copy)]
struct BootRecord {
    pub bdb_oem_id: BootRecordOEM,
    pub bdb_bytes_per_sector: u16,
    pub bdb_sectors_per_cluster: u8,
    pub bdb_reserved_sectors: u16,
    pub bdb_fat_count: u8,
    pub bdb_dir_entries_count: u16,
    pub bdb_total_sectors: u16,
    pub bdb_sectors_per_fat: u16,
    pub bdb_sectors_per_track: u16,
    pub bdb_hidden_sectors: u32,

    pub ebr_volume_id: BootRecordVolumeId,
    pub ebr_volume_label: FATLabel,
    pub ebr_system_id: BootRecordSystemId,
}

impl From<&[u8]> for BootRecord {
    fn from(value: &[u8]) -> Self {
        // https://averstak.tripod.com/fatdox/bootsec.htm

        let mut bdb_oem_id = BootRecordOEM([0; 8]);
        bdb_oem_id.0.copy_from_slice(&value[0x03..0x0B]);
        let bdb_bytes_per_sector = u16_from_slice(&value[0x0B..0x0D]);
        let bdb_sectors_per_cluster = value[0x0D];
        let bdb_reserved_sectors = u16_from_slice(&value[0x0E..0x10]);
        let bdb_fat_count = value[0x10];
        let bdb_dir_entries_count = u16_from_slice(&value[0x11..0x13]);
        let bdb_total_sectors = u16_from_slice(&value[0x13..0x15]);
        let bdb_sectors_per_fat = u16_from_slice(&value[0x16..0x18]);
        let bdb_sectors_per_track = u16_from_slice(&value[0x18..0x1A]);
        let bdb_hidden_sectors = u32_from_slice(&value[0x1C..0x20]);

        let mut ebr_volume_id = BootRecordVolumeId([0; 4]);
        ebr_volume_id.0.copy_from_slice(&value[0x27..0x2B]);
        let mut ebr_volume_label = FATLabel([0; 11]);
        ebr_volume_label.0.copy_from_slice(&value[0x2B..0x36]);
        let mut ebr_system_id = BootRecordSystemId([0; 8]);
        ebr_system_id.0.copy_from_slice(&value[0x36..0x3E]);

        Self {
            bdb_oem_id,
            bdb_bytes_per_sector,
            bdb_sectors_per_cluster,
            bdb_reserved_sectors,
            bdb_fat_count,
            bdb_dir_entries_count,
            bdb_total_sectors,
            bdb_sectors_per_fat,
            bdb_sectors_per_track,
            bdb_hidden_sectors,
            ebr_volume_id,
            ebr_volume_label,
            ebr_system_id,
        }
    }
}

pub struct FATFile<'a> {
    start_cluster: u16,
    size_bytes: u32,
    driver: &'a mut FATDriver,
}

fn sector_of_fat(cluster: u16) -> u16 {
    (cluster * 12) / (SECTOR_SIZE as u16 * 8)
}

fn section_of_fat(sector: u16) -> u16 {
    sector / FAT_DRIVER_FAT_BUFFER_SECTORS
}

impl<'a> FATFile<'a> {
    pub fn read(&mut self, buffer: &mut [u8]) -> Result<usize, FATDriverError> {
        let mut bytes_read = 0;
        let mut bytes_remaining = self.size_bytes.min(buffer.len() as u32);

        let mut current_cluster = self.start_cluster;
        let mut previous_fat_section = section_of_fat(sector_of_fat(current_cluster));
        let sectors_per_cluster = self.driver.boot_record.bdb_sectors_per_cluster as usize;
        let clusters_per_buffer = (FAT_DRIVER_FAT_BUFFER_SECTORS * SECTOR_SIZE as u16) / 12;

        if self.driver.boot_record.bdb_sectors_per_cluster > 2 {
            return Err(FATDriverError::UnsupportedSectorsPerClusterError);
        }

        self.load_fat_section(previous_fat_section)?;

        while bytes_read < self.size_bytes {
            self.load_cluster(current_cluster)?;

            // Copy the data into the buffer
            let bytes_to_copy = (bytes_remaining as usize).min(SECTOR_SIZE * sectors_per_cluster);
            for b in 0..bytes_to_copy {
                let byte = unsafe { *FAT_DRIVER_FILE_BUFFER_PTR.offset(b as isize) };
                buffer[bytes_read as usize + b] = byte;
            }
            bytes_read += bytes_to_copy as u32;
            bytes_remaining = (self.size_bytes - bytes_read).min(buffer.len() as u32);

            let new_fat_sector = sector_of_fat(current_cluster);
            let new_fat_section = section_of_fat(new_fat_sector);

            if new_fat_section != previous_fat_section {
                println!("Loading new section!");
                self.load_fat_section(new_fat_section)?;

                loop {}

                previous_fat_section = new_fat_section;
            }

            let local_cluster = current_cluster - previous_fat_section * clusters_per_buffer;
            let read_offset = local_cluster % 2 != 0;
            let local_index = (local_cluster * 3) / 2;

            let data_ptr =
                unsafe { FAT_DRIVER_FAT_BUFFER_PTR.offset(local_index as isize) } as *const u16;
            let data = unsafe { data_ptr.read_unaligned() };

            let next_cluster = if read_offset {
                ((data & 0xFF00) >> 4) | ((data & 0xF0) >> 4)
            } else {
                data & 0x0FFF
            };

            // Any of these are supposed to indicate an end of chain marker (there is no more file)
            if next_cluster == 0
                || next_cluster == 1
                || next_cluster == 0xFF0
                || next_cluster >= 0xFF8
            {
                if bytes_read != self.size_bytes {
                    return Err(FATDriverError::InvalidFileSizeError);
                }
            }

            current_cluster = next_cluster;
        }

        Ok(bytes_read as usize)
    }

    // Returns the LBA but also the number of sectors in a cluster
    fn cluster_to_lba(&self, cluster: u16) -> (u32, u32) {
        let sectors_per_cluster = self.driver.boot_record.bdb_sectors_per_cluster;

        let lba = self.driver.data_region_start_sector as u32
            + (cluster as u32 - 2) * sectors_per_cluster as u32;

        (lba, sectors_per_cluster as u32)
    }

    fn load_cluster(&mut self, cluster: u16) -> Result<(), FATDriverError> {
        let (lba, sectors_to_read) = self.cluster_to_lba(cluster);

        self.driver
            .disk
            .read_sectors(lba, sectors_to_read, FAT_DRIVER_FILE_BUFFER_PTR)
            .map_err(|_| FATDriverError::DiskReadError)
    }

    /// This assumes that in FAT12 all FATs are divisible by 3 sectors, which would make sense
    fn load_fat_section(&mut self, section: u16) -> Result<(), FATDriverError> {
        self.driver
            .disk
            .read_sectors(
                (self.driver.fat_start_sector + section * FAT_DRIVER_FAT_BUFFER_SECTORS).into(),
                FAT_DRIVER_FAT_BUFFER_SECTORS.into(),
                FAT_DRIVER_FAT_BUFFER_PTR,
            )
            .map_err(|_| FATDriverError::DiskReadError)
    }
}

pub struct FATDriver {
    disk: Disk,
    boot_record: BootRecord,
    fat_start_sector: u16,
    fat_size_in_sectors: u16,
    root_dir_start_sector: u16,
    root_dir_size_in_sectors: u16,
    data_region_start_sector: u16,
}

impl FATDriver {
    pub fn new(mut disk: Disk) -> Result<Self, DiskReadError> {
        disk.read_sector(0, FAT_DRIVER_BOOT_SECTOR_PTR)?;

        let boot_record_buffer_slice = unsafe {
            core::slice::from_raw_parts(FAT_DRIVER_BOOT_SECTOR_PTR as *const u8, SECTOR_SIZE)
        };

        let boot_record = BootRecord::from(boot_record_buffer_slice);

        let fat_size_in_sectors =
            boot_record.bdb_fat_count as u16 * boot_record.bdb_sectors_per_fat;

        let root_dir_start_sector = boot_record.bdb_reserved_sectors + fat_size_in_sectors;
        let root_dir_size_in_bytes =
            boot_record.bdb_dir_entries_count * (size_of::<DirEntry>() as u16);
        // This calculation rounds up to the nearest whole sector, which is how
        // the data is stored if it doesn't fit neatly
        let root_dir_size_in_sectors =
            (root_dir_size_in_bytes + SECTOR_SIZE as u16 - 1) / SECTOR_SIZE as u16;

        Ok(Self {
            disk,
            boot_record,
            fat_start_sector: boot_record.bdb_reserved_sectors,
            fat_size_in_sectors,
            root_dir_start_sector,
            root_dir_size_in_sectors,
            data_region_start_sector: root_dir_start_sector + root_dir_size_in_sectors,
        })
    }

    pub fn open_file(&mut self, name: &FileName) -> Result<FATFile, FATDriverError> {
        let entry = self.find_entry_in_root(name)?;

        if !entry.is_file() {
            return Err(FATDriverError::FileNotFoundError);
        }

        println!("{}", name.as_str());

        Ok(FATFile {
            start_cluster: entry.first_cluster_low(),
            size_bytes: entry.file_size(),
            driver: self,
        })
    }

    fn find_entry_in_root(&mut self, name: &FileName) -> Result<DirEntry, FATDriverError> {
        let entries_per_buffer: u16 =
            self.boot_record.bdb_dir_entries_count / size_of::<DirEntry>() as u16;

        // Load the very beginning of the root directory
        self.load_root_directory(0)?;

        let mut local_index: isize = 0;

        for i in 0..self.boot_record.bdb_dir_entries_count {
            if local_index > entries_per_buffer as isize {
                // Go to the next section of the root directory
                let offset = i / entries_per_buffer;
                self.load_root_directory(offset)?;
                local_index = 0;
            }

            let entry_ptr =
                unsafe { (FAT_DRIVER_ROOT_DIR_BUFFER_PTR as *const DirEntry).offset(local_index) };
            let entry = unsafe { entry_ptr.as_ref().unwrap() };

            if entry.is_end() {
                break;
            }

            if entry.is_dot() || entry.is_erased() || entry.is_volume_id() {
                local_index += 1;
                continue;
            }

            if &entry.name == name {
                return Ok(*entry);
            }

            local_index += 1;
        }

        Err(FATDriverError::FileNotFoundError)
    }

    fn load_root_directory(&mut self, offset: u16) -> Result<(), FATDriverError> {
        self.disk
            .read_sectors(
                (self.root_dir_start_sector + offset).into(),
                FAT_DRIVER_ROOT_DIR_BUFFER_SECTORS.into(),
                FAT_DRIVER_ROOT_DIR_BUFFER_PTR,
            )
            .map_err(|_| FATDriverError::DiskReadError)
    }

    pub fn volume_label(&self) -> &str {
        self.boot_record.ebr_volume_label.as_str()
    }
}

fn u16_from_slice(input: &[u8]) -> u16 {
    let mut u16_buffer: [u8; 2] = [0; 2];
    u16_buffer.copy_from_slice(input);
    u16::from_ne_bytes(u16_buffer)
}

fn u32_from_slice(input: &[u8]) -> u32 {
    let mut u32_buffer: [u8; 4] = [0; 4];
    u32_buffer.copy_from_slice(input);
    u32::from_ne_bytes(u32_buffer)
}
