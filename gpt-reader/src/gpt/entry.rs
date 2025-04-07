use bin_tools::{read_into_array, read_u16_le, read_u64_le};

use super::guids::GUID;

#[derive(Debug, Clone, Copy)]
pub struct PartitionEntry {
    /// offset 0x00
    partition_type_guid: GUID,
    /// offset 0x10
    partition_guid: GUID,
    /// offset 0x20
    first_lba: u64,
    /// offset 0x28
    last_lba: u64,
    /// offset 0x30
    attribute_flags: u64,
    /// offset 0x38
    partition_name: [u16; 36],
}

impl PartitionEntry {
    pub fn read(buffer: &[u8]) -> Self {
        Self {
            partition_type_guid: GUID::from(read_into_array(buffer, 0x00)),
            partition_guid: GUID::from(read_into_array(buffer, 0x10)),
            first_lba: read_u64_le(buffer, 0x20),
            last_lba: read_u64_le(buffer, 0x28),
            attribute_flags: read_u64_le(buffer, 0x30),
            partition_name: read_partition_name(buffer, 0x38),
        }
    }

    pub fn partition_type_guid(&self) -> &GUID {
        &self.partition_type_guid
    }

    pub fn partition_guid(&self) -> &GUID {
        &self.partition_guid
    }

    pub fn first_lba(&self) -> u64 {
        self.first_lba
    }

    pub fn last_lba(&self) -> u64 {
        self.last_lba
    }

    pub fn sectors(&self) -> u64 {
        self.last_lba - self.first_lba + 1
    }

    pub fn attribute_flags(&self) -> u64 {
        self.attribute_flags
    }

    pub fn name_str(&self) -> String {
        let mut name_end = 0;

        for i in 0..self.partition_name.len() {
            if self.partition_name[i] != 0 {
                name_end = i;
            } else {
                break;
            }
        }

        String::from_utf16_lossy(&self.partition_name[..name_end])
    }
}

fn read_partition_name(buffer: &[u8], offset: usize) -> [u16; 36] {
    let mut buf = [0u16; 36];

    for i in 0..36 {
        buf[i] = read_u16_le(buffer, offset + (i * 2));
    }

    buf
}
