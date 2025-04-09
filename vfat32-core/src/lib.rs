#![no_std]

pub mod entry;
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

fn read_padded_str<const N: usize>(buffer: &[u8], offset: usize) -> [u8; N] {
    let mut label = [0; N];

    for i in 0..N {
        label[i] = buffer[offset + i];
    }

    label
}
