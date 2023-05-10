use core::mem::size_of;

use crate::{
    disk::{Disk, DiskReadError, SECTOR_SIZE},
    print, println,
};

const FAT_DRIVER_BOOT_SECTOR_PTR: *mut u8 = 0x7c00 as *mut u8;

const FAT_DRIVER_ROOT_DIR_BUFFER_PTR: *mut u8 = 0x8000 as *mut u8;
const FAT_DRIVER_ROOT_DIR_BUFFER_SECTORS: u16 = 2;

const FAT_DRIVER_FAT_BUFFER_PTR: *mut u8 = 0x8400 as *mut u8;
const FAT_DRIVER_FAT_BUFFER_SECTORS: u16 = 2;

const FAT_DRIVER_FILE_BUFFER_PTR: *mut u8 = 0x8800 as *mut u8;
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
}

#[derive(Clone, Copy)]
#[repr(C, packed)]
struct FATLabel([u8; 11]);

impl FATLabel {
    pub fn as_str(&self) -> &str {
        unsafe { core::str::from_utf8_unchecked(&self.0) }
    }
}

#[derive(Clone, Copy)]
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
                let mut name_buf = &mut buf[0..8];

                if index >= name_buf.len() {
                    return Err(FileNameError::NameTooLongError);
                }

                name_buf[index] = c_uppercase;
            } else {
                let mut ext_buf = &mut buf[8..];

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
}

#[repr(C, packed)]
struct BootRecord {
    bdb_boot_jump: [u8; 3],
    bdb_oem_id: [u8; 8],
    bdb_bytes_per_sector: u16,
    bdb_sectors_per_cluster: u8,
    bdb_reserved_sectors: u16,
    bdb_fat_count: u8,
    bdb_dir_entries_count: u16,
    bdb_total_sectors: u16,
    bdb_media_descriptor_type: u8,
    bdb_sectors_per_fat: u16,
    bdb_sectors_per_track: u16,
    bdb_head_count: u16,
    bdb_hidden_sectors: u32,
    bdb_large_sectors: u32,

    ebr_drive_number: u8,
    __reserved: u8,
    ebr_signature: u8,
    ebr_volume_id: [u8; 4],
    ebr_volume_label: FATLabel,
    ebr_system_id: [u8; 8],
    //
    // code and magic number
}

impl BootRecord {
    fn sectors_per_fat(&self) -> u16 {
        let sectors_per_fat_ptr = core::ptr::addr_of!(self.bdb_sectors_per_fat);

        unsafe { sectors_per_fat_ptr.read_unaligned() }
    }
}

pub struct FATFile<'a> {
    start_cluster: u16,
    current_cluster: u16,
    size_bytes: u32,
    driver: &'a mut FATDriver,
}

impl<'a> FATFile<'a> {
    pub fn read(&mut self, buffer: &mut [u8]) -> Result<usize, FATDriverError> {
        let fat_entries_per_buffer =
            12.0 / (FAT_DRIVER_FAT_BUFFER_SECTORS * SECTOR_SIZE as u16) as f32;

        let mut bytes_read = 0;
        let mut bytes_remaining = self.size_bytes - bytes_read;
        let mut fat_offset = 0;

        let mut local_index = 0;
        let mut read_offset = false;

        // Load the beginning of the FAT
        self.load_fat(0)?;

        for _ in 0..8 {
            let data_ptr =
                unsafe { FAT_DRIVER_FAT_BUFFER_PTR.offset(local_index as isize) } as *const u16;
            let data = unsafe { data_ptr.read_unaligned() };

            let cluster = if read_offset {
                ((data & 0xFF00) >> 4) | ((data & 0xF0) >> 4)
            } else {
                data & 0x0FFF
            };

            print!("{:03x} ", cluster);

            if read_offset {
                local_index += 1;
            }

            local_index += 1;
            read_offset = !read_offset;
        }

        return Ok(0);

        while bytes_remaining > 0 {
            if local_index >= fat_entries_per_buffer as u16 {
                self.load_fat(fat_offset)?;
            }
        }

        Ok(bytes_read as usize)
    }

    fn load_fat(&mut self, offset: u16) -> Result<(), FATDriverError> {
        self.driver
            .disk
            .read_sectors(
                (self.driver.fat_start_sector + offset).into(),
                FAT_DRIVER_FAT_BUFFER_SECTORS.into(),
                FAT_DRIVER_FAT_BUFFER_PTR,
            )
            .map_err(|_| FATDriverError::DiskReadError)
    }
}

pub struct FATDriver {
    disk: Disk,
    boot_record: &'static BootRecord,
    fat_start_sector: u16,
    fat_size_in_sectors: u16,
    root_dir_start_sector: u16,
    root_dir_size_in_sectors: u16,
    data_region_start_sector: u16,
}

impl FATDriver {
    pub fn new(mut disk: Disk) -> Result<Self, DiskReadError> {
        disk.read_sector(0, FAT_DRIVER_BOOT_SECTOR_PTR)?;

        let boot_record_ptr = FAT_DRIVER_BOOT_SECTOR_PTR as *const BootRecord;

        // SAFETY: The pointer will never be null, we set it in a constant
        //   If the disk was somehow misread, then if all the bytes are junk, well
        //   not much we can do about that.
        let boot_record = unsafe { boot_record_ptr.as_ref().unwrap() };

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

        Ok(FATFile {
            start_cluster: entry.first_cluster_low,
            current_cluster: entry.first_cluster_low,
            size_bytes: entry.file_size,
            driver: self,
        })
    }

    fn find_entry_in_root(&mut self, name: &FileName) -> Result<DirEntry, FATDriverError> {
        let entries_per_buffer: u16 =
            self.boot_record.bdb_dir_entries_count / size_of::<DirEntry>() as u16;

        // Load the very beginning of the root directory
        self.load_root_directory(0)?;

        let first_entry_ptr = FAT_DRIVER_ROOT_DIR_BUFFER_PTR as *const DirEntry;
        let first_entry = unsafe { first_entry_ptr.as_ref().unwrap() };

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
