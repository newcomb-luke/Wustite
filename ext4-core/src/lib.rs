#![no_std]

use core::fmt::Debug;
use core::fmt::Display;

pub mod superblock;
pub mod inode;
pub mod groups;

pub const EXT4_MAGIC: u16 = 0xEF53;

#[derive(Debug, Clone, Copy)]
pub enum Error {
    BufferSizeTooSmall(u32)
}

impl Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::BufferSizeTooSmall(size) => {
                write!(f, "Buffer size too small, was only {size} bytes.")
            }
        }
    }
}
