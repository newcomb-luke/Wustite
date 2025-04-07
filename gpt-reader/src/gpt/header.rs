#![allow(dead_code)]

use bin_tools::{read_into_array, read_u32_le, read_u64_le};

use super::guids::GUID;

const GPT_SIGNATURE: [u8; 8] = [b'E', b'F', b'I', b' ', b'P', b'A', b'R', b'T'];

#[derive(Debug, Clone, Copy)]
pub struct PartitionTableHeader {
    /// offset 0x00
    signature: [u8; 8],
    /// offset 0x08
    revision_number: u32,
    /// offset 0x0c
    header_size: u32,
    /// offset 0x10
    crc_32: u32,
    /// offset 0x18
    current_lba: u64,
    /// offset 0x20
    backup_lba: u64,
    /// offset 0x28
    first_usable_lba: u64,
    /// offset 0x30
    last_usable_lba: u64,
    /// offset 0x38
    disk_guid: GUID,
    /// offset 0x48
    entries_starting_lba: u64,
    /// offset 0x50
    num_partitions: u32,
    /// offset 0x54
    partition_entry_size: u32,
    /// offset 0x58
    partition_entries_crc_32: u32,
}

impl PartitionTableHeader {
    pub fn read(buffer: &[u8]) -> Self {
        Self {
            signature: read_into_array(buffer, 0x00),
            revision_number: read_u32_le(buffer, 0x08),
            header_size: read_u32_le(buffer, 0x0c),
            crc_32: read_u32_le(buffer, 0x10),
            current_lba: read_u64_le(buffer, 0x18),
            backup_lba: read_u64_le(buffer, 0x20),
            first_usable_lba: read_u64_le(buffer, 0x28),
            last_usable_lba: read_u64_le(buffer, 0x30),
            disk_guid: GUID::from(read_into_array(buffer, 0x38)),
            entries_starting_lba: read_u64_le(buffer, 0x48),
            num_partitions: read_u32_le(buffer, 0x50),
            partition_entry_size: read_u32_le(buffer, 0x54),
            partition_entries_crc_32: read_u32_le(buffer, 0x58),
        }
    }

    pub fn is_signature_valid(&self) -> bool {
        self.signature == GPT_SIGNATURE
    }

    pub fn guid(&self) -> &GUID {
        &self.disk_guid
    }

    pub fn first_usable_lba(&self) -> u64 {
        self.first_usable_lba
    }

    pub fn last_usable_lba(&self) -> u64 {
        self.last_usable_lba
    }

    pub fn partition_table_entries_start_lba(&self) -> u64 {
        self.entries_starting_lba
    }

    pub fn num_partition_table_entries(&self) -> u32 {
        self.num_partitions
    }

    pub fn partition_table_entry_size(&self) -> u32 {
        self.partition_entry_size
    }
}
