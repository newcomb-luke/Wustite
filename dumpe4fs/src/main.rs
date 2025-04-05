use std::{fs::File, io::{self, Seek, SeekFrom, Read}, path::PathBuf};
use clap::Parser;

use ext4_core::{groups::GroupDescriptor, inode::Inode, superblock::SuperBlock};

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(value_name = "FILE")]
    file: PathBuf,
}

fn main() {
    let args = Args::parse();

    println!("dumpe4fs {VERSION}");

    match File::open(&args.file) {
        Ok(mut opened) => {
            let mut block = [0u8; 1024];

            if let Err(e) = read_block(&mut opened, 1, 1024, &mut block) {
                eprintln!("Error reading file: {e}");
                return;
            }

            println!("sizeof: {}", core::mem::size_of::<SuperBlock>());

            match SuperBlock::read(&block) {
                Ok(superblock) => {

                    if superblock.magic() != ext4_core::EXT4_MAGIC {
                        eprintln!("Bad magic number in superblock while trying to open {}", &args.file.display());
                        eprintln!("Couldn't find valid filesystem superblock.");
                        return;
                    }

                    println!("Filesystem volume name:   {}", superblock.volume_label());
                    println!("Last mounted on:          {}", superblock.last_mounted().unwrap_or("<not available>"));
                    println!("Filesystem UUID:          {}", superblock.filesystem_uuid());
                    println!("Filesystem magic number:  0x{:04X}", superblock.magic());
                    println!("Filesystem revision #:    {}", superblock.filesystem_revision());
                    println!("Filesystem features:      {}", "");
                    println!("Filesystem flags:         {}", "");
                    println!("Default mount options:    {}", "");
                    println!("Filesystem state:         {:016b}", superblock.filesystem_state().raw_value());
                    println!("Filesystem clean:         {}", superblock.filesystem_state().cleanly_unmounted());
                    println!("Filesystem errors:        {}", superblock.filesystem_state().errors_detected());
                    println!("Filesystem orphans:       {}", superblock.filesystem_state().orphans_being_recovered());
                    println!("Errors behavior:          {}", "");
                    println!("Filesystem OS type:       {}", superblock.creator_os());
                    println!("Group descriptor size:    {}", superblock.group_descriptor_size());

                    let mut gdt_block = [0u8; 1024];

                    if let Err(e) = read_block(&mut opened, 1, 4096, &mut gdt_block) {
                        eprintln!("Error reading file: {e}");
                        return;
                    }

                    let first_gd = GroupDescriptor::read(&gdt_block[0..64]);

                    println!("{:#?}", first_gd);

                    let mut inode_table_block = [0u8; 1024];

                    if let Err(e) = read_block(&mut opened, u32::from(first_gd.inode_table_block()) as u64, 4096, &mut inode_table_block) {
                        eprintln!("Error reading file: {e}");
                        return;
                    }

                    for i in 0..3 {
                        let start = (i * superblock.inode_size()) as usize;
                        let end = start + (superblock.inode_size() as usize);
                        let inode = Inode::read(&inode_table_block[start..end]);

                        println!("{:#?}", inode);
                    }
                }
                Err(e) => {
                    eprintln!("Error reading filesystem superblock: {e:?}");
                }
            }
        }
        Err(e) => {
            eprintln!("Error opening file: {e}");
        }
    }
}

fn read_block(file: &mut File, lba: u64, block_size: u64, buffer: &mut [u8]) -> io::Result<()> {
    file.seek(SeekFrom::Start((lba * block_size) + 316669952))?;
    file.read(buffer)?;
    Ok(())
}
