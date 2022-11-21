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

fn main(boot_info: &BootInfo) {
    kprintln!("Boot drive number: 0x{:02x}", boot_info.boot_drive);

    for region in boot_info.memory_regions {
        kprintln!(
            "Start: 0x{:x}, end: 0x{:x}, kind: {:?}",
            region.start,
            region.end,
            region.kind
        );
    }
}
