use crate::logln;

use super::{pci::PCIGeneralDevice, write_io_port_u8, write_io_port_u16};

const REG_DATA: u16 = 0;
const REG_ERROR: u16 = 1;
const REG_FEATURES: u16 = 1;
const REG_SECTOR_COUNT: u16 = 2;
const REG_LBA_LO: u16 = 3;
const REG_LBA_MID: u16 = 4;
const REG_LBA_HI: u16 = 5;
const REG_DRIVE: u16 = 6;
const REG_STATUS: u16 = 7;
const REG_COMMAND: u16 = 7;

const REG_ALT_STATUS: u16 = 0;
const REG_CONTROL: u16 = 0;
const REG_DRIVE_ADDRESS: u16 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Drive {
    Master,
    Slave,
}

impl Drive {
    fn value(&self) -> u8 {
        match self {
            Self::Master => 0xA0,
            Self::Slave => 0xB0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct IDEChannel {
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

    fn control_port(&self) -> u16 {
        self.control + REG_CONTROL
    }

    fn drive_select_port(&self) -> u16 {
        self.base + REG_DRIVE
    }

    fn sector_count_port(&self) -> u16 {
        self.base + REG_SECTOR_COUNT
    }

    fn write_control(&mut self, value: u8) {
        unsafe {
            write_io_port_u8(self.control_port(), value);
        }
    }

    fn set_interrupts(&mut self, enabled: bool) {
        self.write_control(if enabled { 0 } else { 1 } << 1);
    }

    fn select_drive(&mut self, drive: Drive) {
        unsafe {
            write_io_port_u8(self.drive_select_port(), drive.value());
        }
    }

    fn set_sector_count(&mut self, value: u16) {
        unsafe {
            write_io_port_u16(self.sector_count_port(), value);
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
