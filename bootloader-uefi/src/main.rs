#![no_std]
#![no_main]

use core::panic;

use common::elf::{ElfFile, FileType};
use uefi::{
    prelude::*,
    table::{
        boot::{AllocateType, MemoryType},
        cfg::ACPI2_GUID,
    },
};
use uefi_services::println;

use crate::{
    filesystem::{find_file, read_file},
    memory::get_memory_map,
};

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

    let kernel_elf = match ElfFile::new_validated(kernel_read_location) {
        Ok(elf) => elf,
        Err(e) => {
            panic!("Failed to verify kernel ELF file: {:?}", e);
        }
    };

    if kernel_elf.file_type() != FileType::Dyn {
        panic!("Only DYN ELF kernel images are supported");
    }

    println!("Kernel was valid");

    let kernel_load_location = boot_services
        .allocate_pages(AllocateType::AnyPages, MemoryType::RESERVED, 20)
        .unwrap();
    let kernel_load_location_size = 20 * 4096;

    let kernel_load_location_slice = unsafe {
        core::slice::from_raw_parts_mut(kernel_load_location as *mut u8, kernel_load_location_size)
    };

    println!("Kernel load location: {kernel_load_location:08x}");

    {
        use core::fmt::Write;
        writeln!(system_table.stdout(), "Hmmm").unwrap();
    }

    unsafe {
        kernel_elf
            .load_dynamic_file(kernel_load_location_slice, system_table.stdout())
            .unwrap();
    }

    // Buys us 5 minutes to look at the output of our horrible code
    loop {}

    let (first_region, num_regions) = get_memory_map(boot_services).unwrap();

    let acpi_rsdp = system_table
        .config_table()
        .iter()
        .find(|e| e.guid == ACPI2_GUID)
        .map(|e| e.address as *const u8)
        .expect("Could not find ACPI table, cannot continue.");

    println!("Address of ACPI RSDP table: {:?}", acpi_rsdp);

    let _ = system_table.exit_boot_services();

    Status::SUCCESS
}
