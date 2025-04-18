use alloc::boxed::Box;
use kernel::SystemError;

use super::pci::PCIDevice;
use super::pci::PCIDeviceSetup;

#[non_exhaustive]
pub enum DriverCreator {
    PCIDriver(Box<dyn PCIDriverCreator>),
}

pub trait PCIDriverCreator {
    fn detect(&self, device: &PCIDevice) -> bool;
    fn create(&self, setup: PCIDeviceSetup) -> Result<Box<dyn PCIDriver>, SystemError>;
}

pub trait PCIDriver {
    fn name(&self) -> &'static str;
}
