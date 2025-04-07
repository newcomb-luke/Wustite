use std::fmt::{Debug, Display};

use bin_tools::{read_u16_be, read_u16_le, read_u32_be, read_u32_le};

pub const EFI_SYSTEM_PARTITION: GUID = GUID([
    0x28, 0x73, 0x2a, 0xc1, 0x1f, 0xf8, 0xd2, 0x11, 0xba, 0x4b, 0x00, 0xa0, 0xc9, 0x3e, 0xc9, 0x3b,
]);
pub const LINUX_FILESYSTEM_DATA: GUID = GUID([
    0xaf, 0x3d, 0xc6, 0x0f, 0x83, 0x84, 0x72, 0x47, 0x8e, 0x79, 0x3d, 0x69, 0xd8, 0x47, 0x7d, 0xe4,
]);

#[derive(Copy, Clone)]
pub struct GUID([u8; 16]);

impl GUID {
    pub fn is_zero(&self) -> bool {
        for i in 0..self.0.len() {
            if self.0[i] != 0 {
                return false;
            }
        }
        true
    }
}

impl PartialEq for GUID {
    fn eq(&self, other: &Self) -> bool {
        for i in 0..self.0.len() {
            if self.0[i] != other.0[i] {
                return false;
            }
        }

        true
    }
}

impl Eq for GUID {}

impl Debug for GUID {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        // write!(f, "{self}")
        write!(f, "{:x?}", self.0)
    }
}

impl Display for GUID {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        write!(
            f,
            "{:08X}-{:04X}-{:04X}-{:04X}-{:08X}{:04X}",
            read_u32_le(&self.0, 0),
            read_u16_le(&self.0, 4),
            read_u16_le(&self.0, 6),
            read_u16_be(&self.0, 8),
            read_u32_be(&self.0, 10),
            read_u16_be(&self.0, 14)
        )
    }
}

impl From<[u8; 16]> for GUID {
    fn from(value: [u8; 16]) -> Self {
        Self(value)
    }
}
