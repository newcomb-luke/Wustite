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

use common::BootInfo;
use drivers::{keyboard::KEYBOARD_BUFFER, serial::SERIAL0};
use x86_64::VirtAddr;

use crate::drivers::{
    pci::{PCIDevice, PCI_SUBSYSTEM},
    video::svga::vmware_svga_2::VMWareSVGADriver
};

fn start_kernel() {
    logln!("------------------------- Kernel loaded -------------------------");

    logln!("Wustite version {}", env!("CARGO_PKG_VERSION"));

    // let acpi_reader = ACPIReader::read(phys_mem_offset).expect("ACPI not found, cannot continue");

    // let available_drives = available_drives();

    // println!("{:#?}", available_drives);

    let pci_devices = PCI_SUBSYSTEM.enumerate_pci_devices();

    let mut vga_driver = None;

    for device in pci_devices {
        logln!("{}", device);

        #[allow(irrefutable_let_patterns)]
        if let PCIDevice::General(device) = device {
            if device.vendor_id() == 0x15AD && device.device_id() == 0x0405 {
                match VMWareSVGADriver::new(device) {
                    Ok(driver) => {
                        vga_driver = Some(driver);
                    }
                    Err(e) => {
                        logln!("Failed to initialize SVGA driver: {e:?}");
                    }
                }
            }
        }
    }

    if vga_driver.is_none() {
        logln!("No SVGA-compatible graphics device found");
        logln!("Failed to initialize graphics");
    }
}

fn initialize_kernel(boot_info: &BootInfo) {
    {
        let mut serial = SERIAL0.lock();
        serial.initialize();
    }

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };

    let memory_regions = unsafe {
        core::slice::from_raw_parts(
            boot_info.memory_regions_start,
            boot_info.memory_regions_count as usize,
        )
    };

    let mut frame_allocator = unsafe { memory::BootInfoFrameAllocator::init(memory_regions) };

    allocator::init_heap(&mut mapper, &mut frame_allocator)
        .expect("Kernel heap initialization failed");

    KEYBOARD_BUFFER.init();
}

fn initialize_platform() {
    crate::gdt::init();
    crate::interrupts::init_idt();
    unsafe { crate::interrupts::PICS.lock().initialize() };
    x86_64::instructions::interrupts::enable();
}

pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}
