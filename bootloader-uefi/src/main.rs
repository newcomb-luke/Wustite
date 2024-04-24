#![no_std]
#![no_main]
#![feature(int_roundings)]

use core::{arch::{asm, x86_64}, panic};

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
use ::x86_64::{
    structures::paging::{PageTable, PageTableFlags, frame::PhysFrame},
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

const KERNEL_STACK_START: u64 = 0x14000000000;
const KERNEL_STACK_SIZE: u64 = 1024 * 128; // 128 KiB should be fine
const KERNEL_STACK_TOP: u64 = KERNEL_STACK_START + KERNEL_STACK_SIZE;
const KERNEL_STACK_PML4T_INDEX: usize = 2;
const KERNEL_STACK_PDPT_INDEX: usize = 256;
const KERNEL_STACK_PAGES_REQUIRED: usize = KERNEL_STACK_SIZE.div_ceil(4096) as usize;

// Upper-half kernel mapping
const KERNEL_VIRTUAL_OFFSET: u64 = 0xC0000000;
const KERNEL_PDPT_INDEX: usize = 3;
const KERNEL_MAX_SIZE: usize = 1024 * 1024 * 2; // 2 MiB

const PHYS_MAP_VIRTUAL_OFFSET: u64 = 0x18000000000;
const PHYS_MAP_PML4T_INDEX: usize = 3;

const MAXIMUM_SUPPORTED_MEMORY: u64 = 0x200000000; // 8 GiB

// We have to make this static so that we can still know how to load it after we switch
// to the kernel's stack
static mut BOOT_INFO_LOCATION: *const BootInfo = core::ptr::null();
static mut KERNEL_ENTRY: u64 = 0;

#[entry]
pub fn main(_image_handle: Handle, mut system_table: SystemTable<Boot>) -> Status {
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

    let (kernel_read_location, kernel_load_location) = read_and_allocate_kernel(boot_services);

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

    let kernel_stack_location = allocate_kernel_stack(boot_services);

    let pml4t_address = init_paging(boot_services, kernel_stack_location, kernel_load_location.as_ptr() as u64).unwrap();

    println!("Paging initalized, PML4T address: {:08x}", pml4t_address);

    let memory_regions = allocate_memory_map_storage(boot_services).unwrap();

    println!("Hopefully this works!");

    // EXITING BOOT SERVICES -- CAN NO LONGER CALL ANY UEFI ROUTINES
    let (_, uefi_memory_map) = system_table.exit_boot_services();

    let (first_region, num_regions) =
        construct_memory_map(memory_regions, uefi_memory_map).unwrap();

    // Finally load the kernel

    unsafe {
        KERNEL_ENTRY = load_kernel(kernel_read_location, kernel_load_location);
        boot_info_location.write_bytes(0, 1);
    }

    unsafe {
        *boot_info_location = BootInfo {
            memory_regions_start: first_region,
            memory_regions_count: num_regions,
            initramfs_address: initramfs_read_location.as_mut_ptr(),
            initramfs_length: initramfs_read_location.len() as u64,
            acpi_rsdp_address: acpi_rsdp,
            physical_memory_offset: PHYS_MAP_VIRTUAL_OFFSET
        };

        BOOT_INFO_LOCATION = boot_info_location;
    }

    switch_paging(pml4t_address);

    // Finally switch to the kernel!
    unsafe { jump_to_kernel() };
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn jump_to_kernel() -> ! {
    // Also sets up the new kernel stack
    
    unsafe {
        asm!(r#"
             xor rbp, rbp
             mov rsp, {}
             mov rdi, {}
             jmp {}
            "#,
            in(reg) KERNEL_STACK_TOP,
            in(reg) BOOT_INFO_LOCATION,
            in(reg) KERNEL_ENTRY);
    }

    loop {}
}

fn switch_paging(pml4t_address: u64) {
    unsafe {
        let flags = ::x86_64::registers::control::Cr3::read().1;
        let phys_addr = PhysAddr::new(pml4t_address);
        let phys_frame = PhysFrame::from_start_address(phys_addr).unwrap();
        ::x86_64::registers::control::Cr3::write(phys_frame, flags);
        ::x86_64::instructions::tlb::flush_all();
    }
}

// We use the boot services allocator to allocate memory for the page tables
// that we will enable once we exit the boot services
fn init_paging(boot_services: &BootServices, kernel_stack_location: u64, kernel_load_location: u64) -> Result<u64, uefi::Error> {
    const NUM_MAP_PAGE_TABLES: usize = 8;
    // One for the PDPT, one for the PDT, and one for the PT
    const NUM_STACK_PAGE_TABLES: usize = 3;
    const NUM_KERNEL_PAGE_TABLES: usize = 1;
    // One for PML4T, an identify mapped PDPT, and a physical PDPT
    const NUM_COMMON_PAGE_TABLES: usize = 3;
    const NUM_PAGE_TABLES_USED: usize = NUM_COMMON_PAGE_TABLES +
        NUM_KERNEL_PAGE_TABLES +
        NUM_STACK_PAGE_TABLES +
        (NUM_MAP_PAGE_TABLES * 2);

    unsafe {
        let page_tables_ptr = boot_services.allocate_pages(
            AllocateType::AnyPages,
            MemoryType::RESERVED,
            NUM_PAGE_TABLES_USED,
        )? as *mut PageTable;

        let pml4t_ptr = page_tables_ptr.add(0);
        let pml4t = pml4t_ptr.as_mut().unwrap();
        pml4t.zero();

        let phys_pdpt_ptr = pml4t_ptr.add(1);
        let phys_pdpt = phys_pdpt_ptr.as_mut().unwrap();
        phys_pdpt.zero();

        let ident_pdpt_ptr = phys_pdpt_ptr.add(1);
        let ident_pdpt = ident_pdpt_ptr.as_mut().unwrap();
        ident_pdpt.zero();

        let stack_pdpt_ptr = ident_pdpt_ptr.add(1);
        let stack_pdpt = stack_pdpt_ptr.as_mut().unwrap();
        stack_pdpt.zero();

        let stack_pdt_ptr = stack_pdpt_ptr.add(1);
        let stack_pdt = stack_pdt_ptr.as_mut().unwrap();
        stack_pdt.zero();

        let stack_pt_ptr = stack_pdt_ptr.add(1);
        let stack_pt = stack_pt_ptr.as_mut().unwrap();
        stack_pt.zero();

        let kernel_pt_ptr = stack_pt_ptr.add(1);
        let kernel_pt = kernel_pt_ptr.as_mut().unwrap();
        kernel_pt.zero();

        let ident_page_directories_start = kernel_pt_ptr.add(1);
        let phys_page_directories_start = ident_page_directories_start.add(NUM_MAP_PAGE_TABLES);

        // Initialize the page map level 4 table
        let pml4t_first = &mut pml4t[0];
        pml4t_first.set_addr(
            PhysAddr::new(ident_pdpt_ptr as u64),
            PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
        );

        let pml4t_stack = &mut pml4t[KERNEL_STACK_PML4T_INDEX];
        pml4t_stack.set_addr(
            PhysAddr::new(stack_pdpt_ptr as u64),
            PageTableFlags::PRESENT | PageTableFlags::WRITABLE
        );

        let pml4t_phys = &mut pml4t[PHYS_MAP_PML4T_INDEX];
        pml4t_phys.set_addr(
            PhysAddr::new(phys_pdpt_ptr as u64),
            PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
        );

        // Initialize the kernel stack page tables
        let pdpt_stack = &mut stack_pdpt[KERNEL_STACK_PDPT_INDEX];
        pdpt_stack.set_addr(
            PhysAddr::new(stack_pdt_ptr as u64),
            PageTableFlags::PRESENT | PageTableFlags::WRITABLE
        );

        let pdt_stack = &mut stack_pdt[0];
        pdt_stack.set_addr(
            PhysAddr::new(stack_pt_ptr as u64),
            PageTableFlags::PRESENT | PageTableFlags::WRITABLE
        );

        // Initialize the kernel stack page table
        for (idx, entry) in stack_pt.iter_mut().enumerate().take(KERNEL_STACK_PAGES_REQUIRED) {
            let addr = kernel_stack_location + (idx * 4096) as u64;
            entry.set_addr(
                PhysAddr::new(addr),
                PageTableFlags::PRESENT | PageTableFlags::WRITABLE
            );
        }

        const TWO_MEGABYTES: u64 = 2 * 1024 * 1024;
        const FOUR_KILOBYTES: u64 = 4 * 1024;
        const NUM_ENTRIES_PER_TABLE: usize = 512;

        // Initialize the identity mapping 2 MiB page directory tables
        for page_table in 0..NUM_MAP_PAGE_TABLES {
            // Initialize the corresponding entries in the PDPT
            let ident_pdpe = &mut ident_pdpt[page_table];
            let pdpe_addr = ident_page_directories_start as u64 + (page_table * 4096) as u64;
            ident_pdpe.set_addr(
                PhysAddr::new(pdpe_addr),
                PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
            );

            let table_ptr = ident_page_directories_start.add(page_table);
            let table = table_ptr.as_mut().unwrap();
            table.zero();

            for (idx, entry) in table.iter_mut().enumerate() {

                if page_table != 0 || idx != 0 {
                    let addr = (page_table * NUM_ENTRIES_PER_TABLE + idx) as u64 * TWO_MEGABYTES;
                    entry.set_addr(
                        PhysAddr::new(addr as u64),
                        PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::HUGE_PAGE
                    );
                } else {
                    entry.set_unused();
                }
            }
        }

        // Initialize the physical memory mapping 2 MiB page directory tables
        for page_table in 0..NUM_MAP_PAGE_TABLES {
            // Initialize the corresponding entries in the PDPT
            let phys_pdpe = &mut phys_pdpt[page_table];
            let pdpe_addr = phys_page_directories_start as u64 + (page_table * 4096) as u64;
            phys_pdpe.set_addr(
                PhysAddr::new(pdpe_addr),
                PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
            );

            let table_ptr = phys_page_directories_start.add(page_table);
            let table = table_ptr.as_mut().unwrap();
            table.zero();

            for (idx, entry) in table.iter_mut().enumerate() {
                let addr = (page_table * NUM_ENTRIES_PER_TABLE + idx) as u64 * TWO_MEGABYTES;
                entry.set_addr(
                    PhysAddr::new(addr as u64),
                    PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::HUGE_PAGE
                );
            }
        }

        // The kernel is actually loaded sort of in the middle of the identity-mapped region
        // So we will have to modify those tables

        // This maps KERNEL_VIRTUAL_OFFSET access to accesses to wherever the kernel was loaded

        let kernel_pdte0 = &mut (ident_page_directories_start.add(KERNEL_PDPT_INDEX)
                               .as_mut().unwrap())[0];
        // Modify the page directory table 0th entry to point to our kernel page table instead
        kernel_pdte0.set_addr(
            PhysAddr::new(kernel_pt_ptr as u64),
            PageTableFlags::PRESENT | PageTableFlags::WRITABLE
        );

        for (idx, entry) in kernel_pt.iter_mut().enumerate() {
            let addr = kernel_load_location + ((idx as u64) * FOUR_KILOBYTES);
            entry.set_addr(
                PhysAddr::new(addr),
                PageTableFlags::PRESENT | PageTableFlags::WRITABLE
            );
        }

        Ok(pml4t_ptr as u64)
    }
}

// This is the high-level function, so when it encounters an unrecoverable error, it just panics
fn read_and_allocate_kernel(boot_services: &BootServices) -> (&'static mut [u8], &'static mut [u8]) {
    let kernel_file = find_file(KERNEL_PATH, boot_services).unwrap();

    println!("Found {}", KERNEL_PATH);

    let kernel_read_location = read_file(kernel_file, boot_services).unwrap();

    println!("Read kernel at: {:?}", kernel_read_location.as_ptr());

    if kernel_read_location.len() > KERNEL_MAX_SIZE {
        panic!("Kernel is larger than 2 MiB. Unsupported");
    }

    let kernel_elf = match ElfFile::new_validated(kernel_read_location) {
        Ok(elf) => elf,
        Err(e) => {
            panic!("Failed to verify kernel ELF file: {:?}", e);
        }
    };

    if kernel_elf.file_type() != FileType::Dyn {
        panic!("Only DYN ELF kernel images are supported");
    }

    println!("Kernel was valid. Entry point {:08x}", kernel_elf.entry_point() + KERNEL_VIRTUAL_OFFSET);

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

    (kernel_read_location, kernel_load_location_slice)
}

// This is the high-level function, so when it encounters an unrecoverable error, it just panics
fn load_kernel(kernel_read_location_slice: &'static mut [u8], kernel_load_location_slice: &'static mut [u8]) -> u64 {
    let kernel_elf = ElfFile::new_validated(kernel_read_location_slice)
        .expect("BOOTLOADER LOGIC ERROR: CALLED LOAD_KERNEL ON NON-ELF FILE");

    unsafe {
        kernel_elf
            .load_dynamic_file(kernel_load_location_slice, KERNEL_VIRTUAL_OFFSET)
            .unwrap();
    }

    kernel_elf.entry_point() + KERNEL_VIRTUAL_OFFSET
}

// This is the high-level function, so when it encounters an unrecoverable error, it just panics
fn allocate_kernel_stack(boot_services: &BootServices) -> u64 {
    let num_pages = KERNEL_STACK_SIZE.div_ceil(4096) as usize;

    let stack_address = boot_services.allocate_pages(
        AllocateType::AnyPages,
        MemoryType::RESERVED,
        num_pages)
        .unwrap() as u64;

    println!("Allocated {} pages of kernel stack starting at {:08x}", num_pages, stack_address);

    stack_address
}
