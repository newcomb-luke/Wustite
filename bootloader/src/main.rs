#![no_std]
#![no_main]

use core::arch::asm;
use core::panic::PanicInfo;

mod a20;
mod gdt;

use a20::enable_a20;
use gdt::GlobalDescriptorTable;

static GDT: GlobalDescriptorTable = GlobalDescriptorTable::unreal();

#[no_mangle]
#[link_section = ".entry"]
pub extern "C" fn entry() -> ! {
    enter_unreal_mode();

    print_str("Reached bootloader!\r\n");

    if let Err(_) = enable_a20() {
        print_str("A20 line failed to enable.\r\n");

        loop {}
    }

    print_str("Woohoo!\r\n");

    loop {}
}

fn print_str(s: &str) {
    for c in s.chars() {
        print_char(c);
    }
}

fn print_char(c: char) {
    unsafe {
        asm!(
            "int 0x10",
            in("al") c as u8,
            in("ah") 0x0eu8,
            in("bx") 0u16
        );
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
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
