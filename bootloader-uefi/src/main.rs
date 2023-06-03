#![no_std]
#![no_main]

use uefi::{
    prelude::*,
    proto::{
        loaded_image::LoadedImage,
        media::{
            file::{Directory, File, FileAttribute, FileMode},
            fs::SimpleFileSystem,
        },
    },
    CStr16,
};
use uefi_services::println;

use crate::filesystem::find_file;

mod filesystem;

const KERNEL_PATH: &str = "KERNEL.O";
const INITRAMFS_PATH: &str = "INITRAMF.IMG";

#[entry]
fn main(_image_handle: Handle, mut system_table: SystemTable<Boot>) -> Status {
    uefi_services::init(&mut system_table).unwrap();

    system_table.stdout().clear().unwrap();

    let boot_services = system_table.boot_services();

    println!("Bootloader started!");

    let kernel_file = find_file(KERNEL_PATH, boot_services).unwrap();

    println!("Found {}", KERNEL_PATH);

    let initramfs_file = find_file(INITRAMFS_PATH, boot_services).unwrap();

    println!("Found {}", INITRAMFS_PATH);

    system_table.boot_services().stall(10_000_000);

    Status::SUCCESS
}
