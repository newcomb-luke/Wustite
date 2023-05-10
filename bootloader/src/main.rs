#![no_std]
#![no_main]
#![allow(unconditional_panic)]

use core::arch::asm;
use core::panic::PanicInfo;

mod a20;
mod disk;
mod gdt;
mod printing;

use a20::enable_a20;
use gdt::GlobalDescriptorTable;

use crate::disk::Disk;

static GDT: GlobalDescriptorTable = GlobalDescriptorTable::unreal();

static mut DRIVE_NUM_PTR: *const u8 = 0x10 as *const u8;

#[no_mangle]
#[link_section = ".entry"]
pub extern "C" fn entry() -> ! {
    enter_unreal_mode();

    let drive_number: u8 = unsafe { *DRIVE_NUM_PTR };

    println!("Reached bootloader!");

    println!("Drive number: 0x{:02x}", drive_number);

    if let Ok(boot_disk) = Disk::from_drive(drive_number) {
        println!("Disk type: {}", boot_disk.drive_type());

        println!("Disk max head: 0x{:02x}", boot_disk.max_head());

        println!("Disk max cylinder: 0x{:04x}", boot_disk.max_cylinder());

        println!("Disk max sector: 0x{:02x}", boot_disk.max_sector());

        if let Err(_) = enable_a20() {
            println!("A20 line failed to enable.");

            loop {}
        }
    } else {
        println!("Failed to read disk parameters.");
    }

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
