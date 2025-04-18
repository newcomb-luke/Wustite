mod device;
mod headers;
mod subsystem;

pub use device::{PCIAddress, PCIDevice, PCIGeneralDevice};
pub use headers::*;
pub use subsystem::*;

use crate::interrupts::VirtualIrq;

#[derive(Clone, Copy)]
pub struct PCIDeviceSetup {
    pub device: PCIDevice,
    pub address: PCIAddress,
    pub irq: Option<VirtualIrq>,
}
