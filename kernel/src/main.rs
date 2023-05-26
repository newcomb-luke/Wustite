#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

extern crate alloc;

mod acpi;
mod allocator;
mod arch;
mod drivers;
mod entry;
mod gdt;
mod interrupts;
mod memory;
mod std;
use alloc::boxed::Box;
use x86_64::VirtAddr;

use crate::{
    acpi::ACPIReader,
    drivers::ata::{Drive, PRIMARY_BUS, SECONDARY_BUS},
    entry::BootInfo,
};

fn init() {
    crate::gdt::init();
    crate::interrupts::init_idt();
    unsafe { crate::interrupts::PICS.lock().initialize() };
    x86_64::instructions::interrupts::enable();
}

fn main(boot_info: &BootInfo) {
    kprintln!("Wustite version {}.\n", env!("CARGO_PKG_VERSION"));

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };

    let mut frame_allocator =
        unsafe { memory::BootInfoFrameAllocator::init(boot_info.memory_regions) };

    allocator::init_heap(&mut mapper, &mut frame_allocator)
        .expect("Kernel heap initialization failed");

    let acpi_reader = ACPIReader::read(phys_mem_offset).expect("ACPI not found, cannot continue");

    // SECONDARY_BUS.disable_interrupts(Drive::Master);
    // let status = SECONDARY_BUS.identify(Drive::Master);

    // kprintln!("Secondary master status: {:#?}", status);

    kprintln!("Didn't crash.");
}

fn sleep() {
    for i in 0..40000 {
        for _ in 0..i {
            x86_64::instructions::nop();
        }
    }
}

pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}
