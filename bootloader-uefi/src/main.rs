#![no_std]
#![no_main]

use uefi::{
    prelude::*,
    proto::{
        loaded_image::LoadedImage,
        media::{
            file::{File, FileAttribute, FileMode},
            fs::SimpleFileSystem,
        },
    },
};
use uefi_services::println;

#[entry]
fn main(_image_handle: Handle, mut system_table: SystemTable<Boot>) -> Status {
    uefi_services::init(&mut system_table).unwrap();

    system_table.stdout().clear().unwrap();

    let boot_services = system_table.boot_services();

    println!("Bootloader started!");

    open_boot_fs(boot_services).unwrap();

    system_table.boot_services().stall(10_000_000);
    Status::SUCCESS
}

fn open_boot_fs(boot_services: &BootServices) -> uefi::Result {
    let loaded_image =
        boot_services.open_protocol_exclusive::<LoadedImage>(boot_services.image_handle())?;

    let mut volume_handle =
        boot_services.open_protocol_exclusive::<SimpleFileSystem>(loaded_image.device())?;

    let mut volume = volume_handle.open_volume()?;

    let mut file_handle = volume.open(
        cstr16!("BOOTX64.EFI"),
        FileMode::Read,
        FileAttribute::empty(),
    )?;

    if file_handle.is_regular_file()? {
        println!("Yep, that's a regular old file.");
    }

    Ok(())
}
