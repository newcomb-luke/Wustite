use std::{
    fs::File,
    io::{self, Read, Seek, SeekFrom},
};

use vfat32_core::{fs_info::FSInfo, record::BootRecord};

fn main() {
    let mut block = [0u8; 512];

    let mut file = std::fs::File::open("test-fat32.img").unwrap();

    if let Err(e) = read_block(&mut file, 0, 512, &mut block) {
        eprintln!("Error reading file: {e}");
        return;
    }

    let boot_record = BootRecord::read(&block);

    println!("{:#?}", boot_record);

    if let Err(e) = read_block(&mut file, 1, 512, &mut block) {
        eprintln!("Error reading file: {e}");
        return;
    }

    let fs_info = FSInfo::read(&block);

    println!("{:#?}", fs_info);

    if !fs_info.is_valid() {
        eprintln!("FSInfo is invalid!");
        return;
    }

    println!("FSInfo is valid");

    let fat_start_sector = boot_record.first_fat_sector() as u64;

    if let Err(e) = read_block(&mut file, fat_start_sector, 512, &mut block) {
        eprintln!("Error reading file: {e}");
        return;
    }

    for row in 0..16 {
        for col in 0..16 {
            let addr = col + row * 16;
            print!("{:02X} ", block[addr]);
        }
        println!();
    }
}

fn read_block(file: &mut File, lba: u64, block_size: u64, buffer: &mut [u8]) -> io::Result<()> {
    file.seek(SeekFrom::Start((lba * block_size) + 0))?;
    file.read(buffer)?;
    Ok(())
}
