use bin_tools::{read_u16_le, read_u32_le};

#[derive(Debug, Clone, Copy)]
pub struct BootRecord {
    // ----- BIOS Parameter Block -----
    /// offset 0x0B
    bytes_per_sector: u16,
    /// offset 0x0D
    sectors_per_cluster: u8,
    /// offset 0x0E
    num_reserved_sectors: u16,
    /// offset 0x10
    num_file_allocation_tables: u8,
    // /// offset 0x11
    // num_root_directory_entries: u16,
    /// offset 0x13
    total_sectors: u16,
    // /// offset 0x15
    // media_descriptor: u8,
    // /// offset 0x18
    // sectors_per_track: u16,
    // /// offset 0x1A
    // num_heads: u16,
    /// offset 0x1C
    num_hidden_sectors: u32,
    /// offset 0x20
    large_total_sectors: u32,
    // ----- Extended Boot Record -----
    /// offset 0x24
    sectors_per_fat: u32,
    /// offset 0x28
    flags: u16,
    /// offset 0x2A
    fat_version: u16,
    /// offset 0x2C
    root_directory_cluster: u32,
    /// offset 0x30
    fs_info_sector: u16,
    /// offset 0x32
    backup_boot_data_sector: u16,
    // /// offset 0x40
    // drive_number: u8,
    /// offset 0x42
    signature: u8, // Must be 0x28 or 0x29
    /// offset 0x43
    volume_serial_number: u32,
    /// offset 0x47
    volume_label: [char; 11],
}

impl BootRecord {
    pub fn read(buffer: &[u8]) -> Self {
        Self {
            bytes_per_sector: read_u16_le(buffer, 0x0B),
            sectors_per_cluster: buffer[0x0D],
            num_reserved_sectors: read_u16_le(buffer, 0x0E),
            num_file_allocation_tables: buffer[0x10],
            // num_root_directory_entries: read_u16_le(buffer, 0x11),
            total_sectors: read_u16_le(buffer, 0x13),
            // media_descriptor: buffer[0x15],
            // sectors_per_track: read_u16_le(buffer, 0x18),
            // num_heads: read_u16_le(buffer, 0x1A),
            num_hidden_sectors: read_u32_le(buffer, 0x1C),
            large_total_sectors: read_u32_le(buffer, 0x20),
            sectors_per_fat: read_u32_le(buffer, 0x24),
            flags: read_u16_le(buffer, 0x28),
            fat_version: read_u16_le(buffer, 0x2A),
            root_directory_cluster: read_u32_le(buffer, 0x2C),
            fs_info_sector: read_u16_le(buffer, 0x30),
            backup_boot_data_sector: read_u16_le(buffer, 0x32),
            // drive_number: buffer[0x40],
            signature: buffer[0x42], // Must be 0x28 or 0x29
            volume_serial_number: read_u32_le(buffer, 0x43),
            volume_label: read_volume_label(buffer, 0x47),
        }
    }

    pub fn bytes_per_sector(&self) -> u16 {
        self.bytes_per_sector
    }

    pub fn first_fat_sector(&self) -> u32 {
        self.num_reserved_sectors as u32
    }

    pub fn num_sectors(&self) -> u32 {
        if self.total_sectors == 0 {
            self.large_total_sectors
        } else {
            self.total_sectors as u32
        }
    }

    pub fn root_directory_cluster(&self) -> u32 {
        self.root_directory_cluster
    }

    pub fn fs_info_sector(&self) -> u32 {
        self.fs_info_sector as u32
    }
}

fn read_volume_label(buffer: &[u8], offset: usize) -> [char; 11] {
    let mut label = ['\0'; 11];

    for i in 0..11 {
        label[i] = buffer[offset + i] as char;
    }

    label
}
