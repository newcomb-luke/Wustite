#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

extern crate alloc;

mod acpi;
mod allocator;
mod drivers;
mod entry;
mod gdt;
mod interrupts;
mod memory;

use acpi::init_acpi;
use common::BootInfo;
use drivers::{
    keyboard::KEYBOARD_BUFFER,
    nvme::NVMEDriver,
    pci::PCIDeviceClass,
    serial::{SERIAL0, initialize_serial},
};
use memory::initialize_memory;

use crate::drivers::pci::{PCI_SUBSYSTEM, PCIDevice};

fn start_kernel(boot_info: &BootInfo) {
    initialize_serial();

    logln!(
        "[info] Hello from Wustite version {}!",
        env!("CARGO_PKG_VERSION")
    );

    initialize_memory(boot_info);
    init_acpi(boot_info);

    KEYBOARD_BUFFER.init();

    let pci_devices = PCI_SUBSYSTEM.enumerate_pci_devices();

    let mut nvme_driver = None;

    for device in pci_devices {
        logln!("{}", device);

        continue;

        #[allow(irrefutable_let_patterns)]
        if let PCIDevice::General(device) = device {
            if device.device_class()
                == PCIDeviceClass::MassStorage(drivers::pci::MassStorageController::NVMController)
            {
                match NVMEDriver::new(device) {
                    Ok(driver) => {
                        nvme_driver = Some(driver);
                    }
                    Err(e) => {
                        logln!("Failed to initialized NVMe driver: {e:?}");
                    }
                }
            }
            // if device.vendor_id() == 0x15AD && device.device_id() == 0x0405 {
            //     match VMWareSVGADriver::new(device) {
            //         Ok(driver) => {
            //             vga_driver = Some(driver);
            //         }
            //         Err(e) => {
            //             logln!("Failed to initialize SVGA driver: {e:?}");
            //         }
            //     }
            // }
            // if device.vendor_id() == 0x8086 && device.device_id() == 0x7010 {
            //     ide_driver = Some(IDEDriver::new(device));
            // }
        }
    }

    // if vga_driver.is_none() {
    //     logln!("No SVGA-compatible graphics device found");
    //     logln!("Failed to initialize graphics");
    // }
}
