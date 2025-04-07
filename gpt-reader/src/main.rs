use std::fs::File;
use std::io::Read;

use gpt_reader::gpt::entry::PartitionEntry;
use gpt_reader::gpt::guids::EFI_SYSTEM_PARTITION;
use gpt_reader::gpt::guids::GUID;
use gpt_reader::gpt::guids::LINUX_FILESYSTEM_DATA;
use gpt_reader::gpt::header::PartitionTableHeader;

const DEFAULT_SECTOR_SIZE: usize = 512;
const ONE_KB: usize = 1024;
const ONE_MB: usize = 1024 * ONE_KB;
const ONE_GB: usize = 1024 * ONE_MB;

fn main() {
    let img_path = "../dumpe4fs/Manjaro-Test2.img";
    let mut file = File::open(img_path).unwrap();

    let size_in_bytes = file.metadata().unwrap().len() as usize;

    let mut first_8_sectors = Box::new([0u8; 1024 * 8]);

    file.read(first_8_sectors.as_mut_slice()).unwrap();

    let table = PartitionTableHeader::read(&first_8_sectors.as_slice()[0x200..]);

    if !table.is_signature_valid() {
        eprintln!("Provided file is not a GPT partitioned disk image");
        return;
    }

    let sector_size = DEFAULT_SECTOR_SIZE;
    let size_in_sectors = size_in_bytes / sector_size;
    let human_size = human_readable_disk_size(size_in_bytes);

    println!("Disk {img_path}: {human_size}, {size_in_bytes} bytes, {size_in_sectors} sectors");
    println!("Units: sectors of 1 * {sector_size} = {sector_size} bytes");
    println!("Sector size (logical/physical): {sector_size} bytes / {sector_size} bytes");
    println!("I/O size (minimal/optimal): {sector_size} bytes / {sector_size} bytes");
    println!("Disklabel type: gpt");

    println!("Disk identifier: {}", table.guid());

    println!();
    println!("Device                          Start      End  Sectors  Size Type");

    let partition_table_start_bytes =
        table.partition_table_entries_start_lba() as usize * sector_size;
    let partition_entries_table = &first_8_sectors[partition_table_start_bytes..];
    let num_entries = table.num_partition_table_entries() as usize;

    for i in 0..num_entries {
        let start = i * table.partition_table_entry_size() as usize;
        let part = PartitionEntry::read(&partition_entries_table[start..]);

        if part.partition_type_guid().is_zero() {
            break;
        }

        let start_width = (format!("{}", part.first_lba()).len()).max(6);
        let end_width = (format!("{}", part.last_lba()).len()).max(8);
        let sectors_width = (format!("{}", part.sectors()).len()).max(8);
        let size_str = human_readable_part_size(part.sectors() as usize * sector_size);
        let size_width = size_str.len().max(5);
        let type_str = type_str(part.partition_type_guid());

        println!(
            "{img_path}{} {:start_width$} {:end_width$} {:sectors_width$} {:>size_width$} {}",
            i + 1,
            part.first_lba(),
            part.last_lba(),
            part.sectors(),
            size_str,
            type_str
        )
    }
}

fn type_str(guid: &GUID) -> &'static str {
    if *guid == EFI_SYSTEM_PARTITION {
        "EFI System"
    } else if *guid == LINUX_FILESYSTEM_DATA {
        "Linux filesystem"
    } else {
        "Unknown"
    }
}

fn human_readable_part_size(size_bytes: usize) -> String {
    if size_bytes >= ONE_GB {
        let num_gigabytes = size_bytes / ONE_GB;
        let leftover = size_bytes % ONE_GB;

        if leftover != 0 {
            let fractional = (num_gigabytes as f32) + (leftover as f32 / ONE_GB as f32);
            format!("{fractional:.1}G")
        } else {
            format!("{num_gigabytes}G")
        }
    } else if size_bytes >= ONE_MB {
        let num_megabytes = size_bytes / ONE_MB;
        let leftover = size_bytes % ONE_MB;

        if leftover != 0 {
            let fractional = (num_megabytes as f32) + (leftover as f32 / ONE_MB as f32);
            format!("{fractional:.1}M")
        } else {
            format!("{num_megabytes}M")
        }
    } else {
        let num_kilobytes = size_bytes / ONE_KB;
        let leftover = size_bytes % ONE_KB;

        if leftover != 0 {
            let fractional = (num_kilobytes as f32) + (leftover as f32 / ONE_KB as f32);
            format!("{fractional:.1}K")
        } else {
            format!("{num_kilobytes}K")
        }
    }
}

fn human_readable_disk_size(size_bytes: usize) -> String {
    if size_bytes >= ONE_GB {
        let num_gigabytes = size_bytes / ONE_GB;
        let leftover = size_bytes % ONE_GB;

        if leftover != 0 {
            let fractional = (num_gigabytes as f32) + (leftover as f32 / ONE_GB as f32);
            format!("{fractional:.2} GiB")
        } else {
            format!("{num_gigabytes} GiB")
        }
    } else if size_bytes >= ONE_MB {
        todo!()
    } else {
        todo!()
    }
}
