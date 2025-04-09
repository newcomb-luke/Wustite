use core::str;

use bin_tools::{read_u16_le, read_u32_le};

use crate::read_padded_str;

pub const DIRECTORY_ENTRY_SIZE: usize = 32;

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum Attributes {
    ReadOnly = 0x01,
    Hidden = 0x02,
    System = 0x04,
    VolumeId = 0x08,
    Directory = 0x10,
    Archive = 0x20,
}

#[derive(Debug, Clone, Copy)]
pub enum DirectoryEntry {
    LFN(LongFileNameEntry),
    Real(RealEntry),
}

impl DirectoryEntry {
    pub fn read(input: &[u8]) -> Self {
        // Attribute field, if it has all lower bits set, it is a long file name entry
        if input[0x0B] == 0x0F {
            Self::LFN(LongFileNameEntry::read(input))
        } else {
            Self::Real(RealEntry::read(input))
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct LongFileNameEntry {
    /// offset 0x00
    sequence_number: u8,
    /// 5 - offset 0x01
    /// 6 - offset 0x0E
    /// 2 - offset 0x1C
    name: [u16; 13],
    /// offset 0x0D
    short_name_checksum: u8,
}

impl LongFileNameEntry {
    pub fn read(input: &[u8]) -> Self {
        Self {
            sequence_number: input[0x00],
            name: Self::read_name(input),
            short_name_checksum: input[0x0D],
        }
    }

    pub fn is_deleted(&self) -> bool {
        self.sequence_number == 0xE5
    }

    pub fn sequence_number(&self) -> u8 {
        self.sequence_number
    }

    pub fn short_name_checksum(&self) -> u8 {
        self.short_name_checksum
    }

    pub fn name(&self) -> impl Iterator<Item = char> {
        char::decode_utf16(self.name.into_iter().filter(|v| *v != 0xFFFF)).filter_map(|e| e.ok())
    }

    fn read_name(input: &[u8]) -> [u16; 13] {
        let name_1: [u16; 5] = Self::read_name_part(input, 0x01);
        let name_2: [u16; 6] = Self::read_name_part(input, 0x0E);
        let name_3: [u16; 2] = Self::read_name_part(input, 0x1C);

        let mut name = [0u16; 13];

        (&mut name[0..5]).copy_from_slice(&name_1);
        (&mut name[5..11]).copy_from_slice(&name_2);
        (&mut name[11..13]).copy_from_slice(&name_3);

        name
    }

    fn read_name_part<const N: usize>(input: &[u8], offset: usize) -> [u16; N] {
        let mut buffer = [0u16; N];

        for i in 0..N {
            buffer[i] = read_u16_le(input, offset + i * 2);
        }

        buffer
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RealEntry {
    /// offset 0x00
    short_name: [u8; 11],
    /// offset 0x0B
    attributes: u8,
    /// offset 0x0C
    entry_case: u8,
    /// offset 0x0D
    creation_time_ms: u8,
    /// offset 0x0E
    creation_time_whole: u16,
    /// offset 0x10
    creation_date: u16,
    /// offset 0x12
    access_date: u16,
    // /// offset 0x14
    // high_cluster_number: u16,
    // /// offset 0x1A
    // lower_cluster_number: u16,
    cluster_number: u32,
    /// offset 0x16
    modified_time: u16,
    /// offset 0x18
    modified_date: u16,
    /// offset 0x1C
    file_size: u32,
}

impl RealEntry {
    pub fn read(input: &[u8]) -> Self {
        let high_cluster_number = read_u16_le(input, 0x14);
        let lower_cluster_number = read_u16_le(input, 0x1A);

        Self {
            short_name: read_padded_str(input, 0x00),
            attributes: input[0x0B],
            entry_case: input[0x0C],
            creation_time_ms: input[0x0D],
            creation_time_whole: read_u16_le(input, 0x0E),
            creation_date: read_u16_le(input, 0x10),
            access_date: read_u16_le(input, 0x12),
            cluster_number: ((high_cluster_number as u32) << 16) | (lower_cluster_number as u32),
            modified_time: read_u16_le(input, 0x16),
            modified_date: read_u16_le(input, 0x18),
            file_size: read_u32_le(input, 0x1C),
        }
    }

    pub fn start_cluster(&self) -> u32 {
        self.cluster_number
    }

    pub fn is_dir(&self) -> bool {
        (self.attributes & (Attributes::Directory as u8)) != 0
    }

    pub fn is_file(&self) -> bool {
        let not_dir = (self.attributes & Attributes::Directory as u8) == 0;
        let not_label = (self.attributes & Attributes::VolumeId as u8) == 0;

        not_dir & not_label
    }

    pub fn is_empty(&self) -> bool {
        self.short_name[0] == 0
    }

    pub fn has_extension(&self) -> bool {
        self.is_file() & (&self.short_name[8..11] != &[b' ', b' ', b' '])
    }

    pub fn name_bytes(&self) -> &[u8; 11] {
        &self.short_name
    }

    pub fn is_name_lowercase(&self) -> bool {
        (&self.entry_case & (1 << 3)) != 0
    }

    pub fn is_extension_lowercase(&self) -> bool {
        (&self.entry_case & (1 << 4)) != 0
    }

    pub fn short_name(&self) -> &str {
        unsafe { str::from_utf8_unchecked(&self.short_name) }
    }

    pub fn file_size(&self) -> u32 {
        self.file_size
    }
}
