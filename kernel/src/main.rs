#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

mod arch;
mod drivers;
mod entry;
mod gdt;
mod interrupts;
mod memory;
mod std;
use crate::entry::BootInfo;

fn init() {
    crate::gdt::init();
    crate::interrupts::init_idt();
    unsafe { crate::interrupts::PICS.lock().initialize() };
    x86_64::instructions::interrupts::enable();
}

fn main(boot_info: &BootInfo) {
    kprintln!("Boot drive number: 0x{:02x}", boot_info.boot_drive);

    // for region in boot_info.memory_regions {
    //     kprintln!(
    //         "Start: 0x{:x}, end: 0x{:x}, kind: {:?}",
    //         region.start,
    //         region.end,
    //         region.kind
    //     );
    // }

    kprintln!("Didn't crash yet!");
}