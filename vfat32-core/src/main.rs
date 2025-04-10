use block_device::{impls::FileBlockDevice, BlockDevice};
use vfat32_core::{fs_info::FSInfo, record::BootRecord};

fn main() {
    let file = std::fs::File::open("test-fat32.img").unwrap();
    let mut block_device = FileBlockDevice::new(file);

    let block_size = block_device.block_size() as usize;
    let mut buffer = vec![0; block_size];

    block_device.read_block(0, &mut buffer).unwrap();

    let boot_record = BootRecord::read(&buffer);

    println!("{:#?}", boot_record);

    block_device.read_block(1, &mut buffer).unwrap();

    let fs_info = FSInfo::read(&buffer);

    println!("{:#?}", fs_info);

    if !fs_info.is_valid() {
        eprintln!("FSInfo is invalid!");
        return;
    }

    println!("FSInfo is valid");

    let fat_start_sector = boot_record.first_fat_sector() as u64;

    block_device
        .read_block(fat_start_sector, &mut buffer)
        .unwrap();

    for row in 0..16 {
        for col in 0..16 {
            let addr = col + row * 16;
            print!("{:02X} ", buffer[addr]);
        }
        println!();
    }

    println!(
        "Root directory cluster: {}",
        boot_record.root_directory_cluster()
    );
}
