#![no_std]

pub mod fs_info;
pub mod record;

use core::fmt::Debug;
use core::fmt::Display;

#[derive(Debug, Clone, Copy)]
pub enum Error {
    BufferSizeTooSmall(u32),
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
