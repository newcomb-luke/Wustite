#![no_std]
#![no_main]

use core::panic;

use uefi::{prelude::*, table::boot::MemoryType};
use uefi_services::{print, println};

use crate::{
    elf::validate_elf,
    filesystem::{find_file, read_file},
    memory::get_memory_map,
};

mod elf;
mod filesystem;
mod memory;

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

    let kernel_read_location = read_file(kernel_file, boot_services).unwrap();

    println!("Loaded kernel at: {:?}", kernel_read_location.as_ptr());

    let initramfs_read_location = read_file(initramfs_file, boot_services).unwrap();

    println!(
        "Initramfs loaded at: {:?}",
        initramfs_read_location.as_ptr()
    );

    if let Err(e) = validate_elf(kernel_read_location) {
        panic!("Failed to verify kernel file: {:?}", e);
    }

    println!("Kernel was a valid ELF binary!");

    let (first_region, num_regions) = get_memory_map(boot_services).unwrap();

    // Buys us 5 minutes to look at the output of our horrible code
    loop {}

    Status::SUCCESS
}
