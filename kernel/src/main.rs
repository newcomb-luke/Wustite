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
use x86_64::VirtAddr;

use crate::{
    drivers::{ata::available_drives, pci::check_pci_device_exists},
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

    // let acpi_reader = ACPIReader::read(phys_mem_offset).expect("ACPI not found, cannot continue");

    let available_drives = available_drives();

    kprintln!("{:#?}", available_drives);

    for bus in 0..1 {
        for device in 0..8 {
            if let Some(header) = check_pci_device_exists(bus, device) {
                kprintln!("Bus {bus}, device {device}:");
                header.print_summary();
            }
        }
    }

    kprintln!("Didn't crash.");
}

pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}
