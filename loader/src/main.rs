#![no_std]
#![no_main]

use core::arch::asm;
use core::panic::PanicInfo;

const KERNEL_LOAD_LOCATION: *mut u8 = 0x00100000 as *mut u8;

mod disk;
mod elf;
mod video;

use video::{BgColor, FgColor, TextBuffer};

pub fn port_out(port: u16, data: u8) {
    unsafe {
        asm!(
        "mov dx, {0:x}", // mov dx, port
        "mov al, {1}",   // mov al, data
        "out dx, al",    // out dx, al
        in(reg_abcd) port,
        in(reg_byte) data,
        in("al") 0u8,
        in("dx") 0u16,
        );
    }
}

pub fn port_in(port: u16) -> u8 {
    let data;

    unsafe {
        asm!(
        "mov dx, {0:x}", // mov dx, port
        "in al, dx",    // out dx, al
        in(reg_abcd) port,
        out("al") data,
        in("dx") 0u16,
        );
    }

    data
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    let mut buffer = TextBuffer::new(FgColor::LightGreen, true, BgColor::Black);

    buffer.putc('0');
    buffer.putc('x');

    loop {}
}

/// This function is called on panic.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
