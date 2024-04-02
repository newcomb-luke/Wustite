#![no_std]
#![no_main]
#![feature(int_roundings)]

use core::panic;

use common::{
    BootInfo,
    KernelEntry,
    memory::MemoryRegion,
    elf::{ElfFile, FileType}
};
use uefi::{
    prelude::*,
    table::{
        boot::{AllocateType, MemoryType},
        cfg::ACPI2_GUID,
    },
};
use uefi_services::println;
use x86_64::{
    structures::paging::{PageTable, PageTableFlags},
    PhysAddr,
};

use crate::{
    filesystem::{find_file, read_file},
    memory::{allocate_memory_map_storage, construct_memory_map},
};

mod filesystem;
mod memory;

const KERNEL_PATH: &str = "kernel.o";
const INITRAMFS_PATH: &str = "initramfs.img";

#[entry]
fn main(_image_handle: Handle, mut system_table: SystemTable<Boot>) -> Status {
    uefi_services::init(&mut system_table).unwrap();

    system_table.stdout().clear().unwrap();

    let boot_services = system_table.boot_services();

    println!("Bootloader started!");

    let initramfs_file = find_file(INITRAMFS_PATH, boot_services).unwrap();

    println!("Found {}", INITRAMFS_PATH);

    let initramfs_read_location = read_file(initramfs_file, boot_services).unwrap();

    println!(
        "Initramfs loaded at: {:?}",
        initramfs_read_location.as_ptr()
    );

    let kernel_entry_point = load_kernel(boot_services);

    println!("Kernel entry point: {:08x}", kernel_entry_point);

    let acpi_rsdp = system_table
        .config_table()
        .iter()
        .find(|e| e.guid == ACPI2_GUID)
        .map(|e| e.address as *const u8)
        .expect("Could not find ACPI table, cannot continue.");

    println!("Address of ACPI RSDP table: {:?}", acpi_rsdp);

    let boot_info_location = boot_services
        .allocate_pool(MemoryType::LOADER_DATA, ::core::mem::size_of::<BootInfo>()).unwrap() as *mut BootInfo;

    println!("Address of Boot Info: {:?}", boot_info_location);

    let memory_regions = allocate_memory_map_storage(boot_services).unwrap();

    println!("Hopefully this works!");

    // EXITING BOOT SERVICES -- CAN NO LONGER CALL ANY UEFI ROUTINES
    let (_, uefi_memory_map) = system_table.exit_boot_services();

    let (first_region, num_regions) =
        construct_memory_map(memory_regions, uefi_memory_map).unwrap();

    unsafe {
        boot_info_location.write_bytes(0, 1);
    }

    unsafe {
        *boot_info_location = BootInfo {
            memory_regions_start: first_region,
            memory_regions_count: num_regions,
            initramfs_address: initramfs_read_location.as_mut_ptr(),
            initramfs_length: initramfs_read_location.len() as u64,
            acpi_rsdp_address: acpi_rsdp,
            physical_memory_offset: 0
        };
    }

    loop {}

    Status::SUCCESS
}

// We use the boot services allocator to allocate memory for the page tables
// that we will enable once we exit the boot services
fn init_paging(boot_services: &BootServices) -> Result<(), uefi::Error> {
    const NUM_PAGE_TABLES_USED: usize = 10;

    unsafe {
        let page_tables_ptr = boot_services.allocate_pages(
            AllocateType::AnyPages,
            MemoryType::RESERVED,
            NUM_PAGE_TABLES_USED,
        )? as *mut PageTable;

        let pml4t = page_tables_ptr.add(0).as_mut().unwrap();
        pml4t.zero();

        let pdpt_ptr = page_tables_ptr.add(1);
        let pdpt = pdpt_ptr.as_mut().unwrap();
        pdpt.zero();

        let pml4t_first = &mut pml4t[0];
        pml4t_first.set_addr(
            PhysAddr::new(pdpt_ptr as u64),
            PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
        );

        let pml4t_phys = &mut pml4t[3];
        pml4t_phys.set_addr(
            PhysAddr::new(0),
            PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
        );
    }

    Ok(())
}

// This is the high-level function, so when it encounters an unrecoverable error, it just panics
fn load_kernel(boot_services: &BootServices) -> u64 {
    let kernel_file = find_file(KERNEL_PATH, boot_services).unwrap();

    println!("Found {}", KERNEL_PATH);

    let kernel_read_location = read_file(kernel_file, boot_services).unwrap();

    println!("Loaded kernel at: {:?}", kernel_read_location.as_ptr());

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

    let kernel_required_bytes = kernel_elf.get_maximum_process_image_size() as usize;
    let kernel_required_pages = kernel_required_bytes.div_ceil(4096);

    let kernel_load_location = boot_services
        .allocate_pages(
            AllocateType::AnyPages,
            MemoryType::RESERVED,
            kernel_required_pages,
        )
        .unwrap();

    // SAFETY: allocate_pages gives us back 4KiB-aligned pointers,
    // and we used the same size as we used to allocate, as we are providing to this function
    let kernel_load_location_slice = unsafe {
        core::slice::from_raw_parts_mut(kernel_load_location as *mut u8, kernel_required_bytes)
    };

    println!("Kernel load location: {kernel_load_location:08x}");

    unsafe {
        kernel_elf
            .load_dynamic_file(kernel_load_location_slice)
            .unwrap();
    }

    kernel_elf.entry_point()
}
