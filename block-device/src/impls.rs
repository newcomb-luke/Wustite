#![cfg(feature = "std")]

use std::{
    fs::File,
    io::{Read, Seek, Write},
};

use crate::BlockDevice;

// Arbitrary, but reasonable
const FILE_BLOCK_SIZE: u64 = 512;

pub struct FileBlockDevice {
    file: File,
    buffer: [u8; FILE_BLOCK_SIZE as usize],
}

impl FileBlockDevice {
    pub fn new(file: File) -> Self {
        Self {
            file,
            buffer: [0u8; FILE_BLOCK_SIZE as usize],
        }
    }
}

impl BlockDevice for FileBlockDevice {
    type Error = std::io::Error;

    fn block_size(&self) -> u64 {
        FILE_BLOCK_SIZE
    }

    fn read_block(&mut self, lba: u64, buffer: &mut [u8]) -> Result<(), Self::Error> {
        let block_size = self.block_size();

        let byte_offset = lba * block_size;

        self.file.seek(std::io::SeekFrom::Start(byte_offset))?;
        self.file.read_exact(&mut self.buffer)?;

        for i in 0..buffer.len() {
            buffer[i] = self.buffer[i];
        }

        Ok(())
    }

    fn write_block(&mut self, lba: u64, buffer: &[u8]) -> Result<(), Self::Error> {
        let block_size = self.block_size();

        let byte_offset = lba * block_size;

        for i in 0..self.buffer.len() {
            self.buffer[i] = buffer[i];
        }

        self.file.seek(std::io::SeekFrom::Start(byte_offset))?;
        self.file.write_all(&self.buffer)?;

        Ok(())
    }
}
