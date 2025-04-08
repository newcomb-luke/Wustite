use bin_tools::read_u32_le;
use block_device::BlockDevice;
use vfat32_core::{fs_info::FSInfo, record::BootRecord};

#[derive(Debug, Clone, Copy)]
pub enum DriverError {
    DiskError,
    FileSystemInvalid,
}

pub struct VFAT32Driver<D>
where
    D: BlockDevice,
{
    block_device: D,
    sector_size: u32,
    boot_record: BootRecord,
    fs_info: FSInfo,
    fat_buffer: [u8; 512],
    root_buffer: [u8; 512],
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
            fat_buffer: [0u8; 512],
            root_buffer: [0u8; 512],
        })
    }

    pub fn find_root_entry(&mut self, name: &str) -> Result<(), DriverError> {
        let root_start_cluster = self.boot_record.root_directory_cluster();

        let (sector, offset) = self.sector_in_fat(root_start_cluster);

        self.block_device
            .read_block(sector, &mut self.fat_buffer)
            .map_err(|_| DriverError::DiskError)?;

        let table_value = read_u32_le(&self.fat_buffer, offset) & 0x0FFFFFFF;

        println!("{:04x}", table_value);

        todo!();
    }

    fn sector_in_fat(&self, cluster: u32) -> (u64, usize) {
        let fat_start_sector = self.boot_record.first_fat_sector() as u64;
        let fat_offset = cluster * 4; // 4 bytes per 32-bit entry

        let sector = fat_start_sector + (fat_offset as u64 / self.sector_size as u64);
        let offset_into_sector = fat_offset % (self.sector_size);

        (sector, offset_into_sector as usize)
    }
}
