#![allow(dead_code)]

use bin_tools::read_u32_le;

#[derive(Debug, Clone, Copy)]
pub struct CHS {
    head: u8,
    cylinder: u16,
    sector: u8,
}

impl CHS {
    pub fn read(buffer: &[u8]) -> Self {
        Self {
            head: buffer[0],
            sector: buffer[1] & 0b00111111,
            cylinder: (buffer[1] >> 6) as u16 & (buffer[2] as u16),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum PartitionType {
    GPT,
    RealPartition,
}

impl From<u8> for PartitionType {
    fn from(value: u8) -> Self {
        match value {
            0xEE => Self::GPT,
            _ => Self::RealPartition,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PartitionEntry {
    /// offset 0x00
    status: u8,
    /// offset 0x01
    chs_start: CHS,
    /// offset 0x04
    ptype: PartitionType,
    /// offset 0x05
    chs_end: CHS,
    /// offset 0x08
    lba_start: u32,
    /// offset 0x0c
    sectors: u32,
}

impl PartitionEntry {
    pub fn read(buffer: &[u8]) -> Self {
        Self {
            status: buffer[0x00],
            chs_start: CHS::read(&buffer[0x01..]),
            ptype: PartitionType::from(buffer[0x04]),
            chs_end: CHS::read(&buffer[0x05..]),
            lba_start: read_u32_le(buffer, 0x08),
            sectors: read_u32_le(buffer, 0x0c),
        }
    }
}
