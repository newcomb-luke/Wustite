use crate::logln;

use super::pci::PCIGeneralDevice;

#[derive(Debug, Clone, Copy)]
pub struct IDEChannel {
    base: u16,
    control: u16,
    bus_master: Option<u16>,
}

impl IDEChannel {
    fn get_primary_channel(interface: &ProgrammingInterface, device: &PCIGeneralDevice) -> Self {
        let bus_master = interface.bus_master().then(|| device.bar4() as u16);

        if interface.is_primary_channel_native_pci_mode() {
            IDEChannel {
                base: device.bar0() as u16,
                control: device.bar1() as u16,
                bus_master,
            }
        } else {
            IDEChannel {
                base: 0x1F0,
                control: 0x3F6,
                bus_master,
            }
        }
    }

    fn get_secondary_channel(interface: &ProgrammingInterface, device: &PCIGeneralDevice) -> Self {
        let bus_master = interface.bus_master().then(|| device.bar4() as u16);

        if interface.is_primary_channel_native_pci_mode() {
            IDEChannel {
                base: device.bar2() as u16,
                control: device.bar3() as u16,
                bus_master,
            }
        } else {
            IDEChannel {
                base: 0x170,
                control: 0x376,
                bus_master,
            }
        }
    }
}

pub struct IDEDriver {
    device: PCIGeneralDevice,
    channels: [IDEChannel; 2],
}

impl IDEDriver {
    pub fn new(device: PCIGeneralDevice) -> Self {
        let interface = ProgrammingInterface::from(device.prog_if());

        let channel1 = IDEChannel::get_primary_channel(&interface, &device);
        let channel2 = IDEChannel::get_secondary_channel(&interface, &device);

        logln!("IDE Channel 1: {:#?}", channel1);
        logln!("IDE Channel 2: {:#?}", channel2);

        Self {
            device,
            channels: [channel1, channel1],
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ProgrammingInterface(u8);

impl ProgrammingInterface {
    pub fn is_primary_channel_native_pci_mode(&self) -> bool {
        (self.0 & (1 << 0)) != 0
    }

    pub fn primary_channel_can_change_native_pci_mode(&self) -> bool {
        (self.0 & (1 << 1)) != 0
    }

    pub fn is_secondary_channel_native_pci_mode(&self) -> bool {
        (self.0 & (1 << 2)) != 0
    }

    pub fn secondary_channel_can_change_native_pci_mode(&self) -> bool {
        (self.0 & (1 << 3)) != 0
    }

    pub fn bus_master(&self) -> bool {
        (self.0 & (1 << 7)) != 0
    }
}

impl From<u8> for ProgrammingInterface {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
