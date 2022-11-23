#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

use core::arch::asm;
use core::fmt::Write;

mod arch;
mod drivers;
mod entry;
mod interrupts;
mod memory;
mod std;
use crate::entry::BootInfo;
use drivers::*;
use video::{Color, TextBuffer};

#[inline(never)]
#[no_mangle]
pub extern "C" fn stack_overflow(mut v: u8) -> ! {
    let x = unsafe { (0x0002 as *mut u64).read_volatile() };
    if (v == u8::MAX) {
        v = 0;
    }
    unsafe {
        (0x0001 as *mut u8).write_volatile(v);
    }
    stack_overflow(unsafe { (0x0001 as *mut u8).read_volatile() });
}

fn main(boot_info: &BootInfo) {
    kprintln!("Boot drive number: 0x{:02x}", boot_info.boot_drive);

    loop {}

    stack_overflow(0);

    for region in boot_info.memory_regions {
        kprintln!(
            "Start: 0x{:x}, end: 0x{:x}, kind: {:?}",
            region.start,
            region.end,
            region.kind
        );
    }
}
