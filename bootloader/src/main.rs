#![no_std]
#![no_main]

use core::arch::asm;
use core::panic::PanicInfo;

mod disk;
mod elf;
mod fat;
mod gdt;
mod long_mode;
mod paging;
mod printing;
mod smap;

use elf::load_elf;
use gdt::GlobalDescriptorTable;

use crate::disk::Disk;
use crate::fat::{FATDriver, FileName};
use crate::long_mode::{is_cpuid_available, is_extended_cpuid_available};
use crate::paging::identity_map_mem;
use crate::smap::detect_memory_regions;

// static GDT: GlobalDescriptorTable = GlobalDescriptorTable::unreal();

const DRIVE_NUM_PTR: *mut u8 = 0x7c24 as *mut u8;

const KERNEL_READ_LOCATION: *mut u8 = 0x00020000 as *mut u8;
const KERNEL_READ_LOCATION_SIZE: usize = 0x00050000;
const KERNEL_MAXIUMUM_SIZE: usize = 0x5ffff;

#[no_mangle]
pub extern "C" fn bootloader_entry() -> ! {
    let drive_number = unsafe { *DRIVE_NUM_PTR };

    if !is_cpuid_available() || !is_extended_cpuid_available() {
        println!("Kernel requires x86_64.");
        halt();
    }

    println!("Hello from bootloader!");

    let Ok(mut boot_disk) = Disk::from_drive(drive_number) else {
        println!("Failed to read disk parameters.");
        halt();
    };

    // println!("Read disk parameters");

    let mut fat_driver = match FATDriver::new(boot_disk) {
        Ok(fat_driver) => fat_driver,
        Err(e) => {
            println!("Failed to iniailzize FAT driver: {:?}", e);
            halt();
        }
    };

    // println!("Initialized FAT driver");

    let file_name_str = "kernel.o";

    let file_name = match FileName::try_from(file_name_str) {
        Ok(file_name) => file_name,
        Err(e) => {
            println!(
                "Failed to convert file name {} into 8.3 format",
                file_name_str
            );
            halt();
        }
    };

    let mut file = match fat_driver.open_file(&file_name) {
        Ok(file) => file,
        Err(e) => {
            println!("Failed to open file: {:?}", e);
            halt();
        }
    };

    let kernel_read_location =
        unsafe { core::slice::from_raw_parts_mut(KERNEL_READ_LOCATION, KERNEL_READ_LOCATION_SIZE) };

    let bytes_read = match file.read(kernel_read_location) {
        Ok(bytes_read) => {
            println!("Kernel was {} bytes", bytes_read);

            bytes_read
        }
        Err(e) => {
            println!("Failed to read file: {:?}", e);
            halt();
        }
    };

    if bytes_read > KERNEL_MAXIUMUM_SIZE {
        println!("Kernel size exceeds maximum available.");
    }

    let entry_point = match load_elf(KERNEL_READ_LOCATION) {
        Ok(entry_point) => {
            println!("Loaded kernel. Entry point: {:08x}", entry_point);

            entry_point
        }
        Err(e) => {
            println!("Failed to load ELF file: {:?}", e);
            halt();
        }
    };

    identity_map_mem();

    // Just in case something happened to it
    unsafe { *DRIVE_NUM_PTR = drive_number };

    if let Err(e) = detect_memory_regions() {
        println!("Error detecting memory: {:?}", e);
    }

    halt();
}

fn halt() -> ! {
    loop {
        unsafe {
            asm!("cli");
        }
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);

    loop {}
}
