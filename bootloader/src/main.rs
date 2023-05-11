#![no_std]
#![no_main]

use core::arch::asm;
use core::panic::PanicInfo;

mod a20;
mod disk;
mod fat;
mod gdt;
mod printing;

use a20::enable_a20;
use gdt::GlobalDescriptorTable;

use crate::disk::Disk;
use crate::fat::{FATDriver, FileName};

static GDT: GlobalDescriptorTable = GlobalDescriptorTable::unreal();

const DRIVE_NUM_PTR: *const u8 = 0x10 as *const u8;

const KERNEL_READ_LOCATION: *mut u8 = 0x00020000 as *mut u8;
const KERNEL_READ_LOCATION_SIZE: usize = 0x00050000;

#[no_mangle]
#[link_section = ".entry"]
pub extern "C" fn entry() -> ! {
    enter_unreal_mode();

    let drive_number: u8 = unsafe { *DRIVE_NUM_PTR };

    println!("Reached bootloader!");

    let boot_disk = if let Ok(boot_disk) = Disk::from_drive(drive_number) {
        boot_disk
    } else {
        println!("Failed to read disk parameters.");
        loop {}
    };

    let mut fat_driver = match FATDriver::new(boot_disk) {
        Ok(fat_driver) => fat_driver,
        Err(e) => {
            println!("Failed to iniailzize FAT driver: {:?}", e);
            loop {}
        }
    };

    let file_name_str = "test.txt";

    let file_name = match FileName::try_from(file_name_str) {
        Ok(file_name) => file_name,
        Err(e) => {
            println!(
                "Failed to convert file name {} into 8.3 format",
                file_name_str
            );

            loop {}
        }
    };

    let mut file = match fat_driver.open_file(&file_name) {
        Ok(file) => file,
        Err(e) => {
            println!("Failed to open file: {:?}", e);

            loop {}
        }
    };

    let kernel_read_location =
        unsafe { core::slice::from_raw_parts_mut(KERNEL_READ_LOCATION, KERNEL_READ_LOCATION_SIZE) };

    let bytes_read = match file.read(kernel_read_location) {
        Ok(bytes_read) => {
            println!("Read {} bytes", bytes_read);

            bytes_read
        }
        Err(e) => {
            println!("Failed to read file: {:?}", e);

            loop {}
        }
    };

    let text = core::str::from_utf8(&kernel_read_location[0..bytes_read]).unwrap();

    println!("{}", text);

    // if let Err(_) = enable_a20() {
    //     println!("A20 line failed to enable.");

    //     loop {}
    // }

    loop {}
}

fn enter_unreal_mode() {
    let ds: u16;
    let ss: u16;

    unsafe {
        asm!("mov {:x}, ds", out(reg) ds);
        asm!("mov {:x}, ss", out(reg) ss);
    }

    GDT.load();

    unsafe {
        let mut cr0: u32;
        asm!("mov {:e}, cr0", out(reg) cr0);

        // Enter protected mode
        let new_cr0 = cr0 | 1;
        asm!("mov cr0, {:e}", in(reg) new_cr0);

        // Initialize segment registers
        asm!("mov {0:x}, 0x10", "mov ds, {0:x}", "mov ss, {0:x}", out(reg) _);

        // Enter back into real mode
        asm!("mov cr0, {:e}", in(reg) cr0);

        asm!("mov ds, {:x}", in(reg) ds);
        asm!("mov ss, {:x}", in(reg) ss);
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);

    loop {}
}
