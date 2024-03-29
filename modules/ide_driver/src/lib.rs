#![no_std]
#![no_main]

use modules_common::DriverType;

#[no_mangle]
static driver_type: DriverType = DriverType::PCIDriver;

#[no_mangle]
pub extern "C" fn __init_pci_driver() -> DriverType {
    return driver_type;
}
