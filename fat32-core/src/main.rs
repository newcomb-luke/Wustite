use std::{
    fs::File,
    io::{self, Read, Seek, SeekFrom},
};

fn main() {
    println!("Hello, world!");
}

fn read_block(file: &mut File, lba: u64, block_size: u64, buffer: &mut [u8]) -> io::Result<()> {
    file.seek(SeekFrom::Start((lba * block_size) + 316669952))?;
    file.read(buffer)?;
    Ok(())
}
