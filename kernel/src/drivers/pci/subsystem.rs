use alloc::{boxed::Box, vec::Vec};
use kernel::SystemError;
use spin::Mutex;

use crate::{
    acpi::acpi_pci_get_routing,
    drivers::{
        pci::PCIDeviceSetup,
        read_io_port_u32,
        register::{DriverCreator, PCIDriver},
        write_io_port_u32,
    },
    kprintln,
};

use super::{
    PCIAddress, PCIDevice, PCIGeneralDevice,
    headers::{PCICommonHeader, PCIDeviceClass, PCIGeneralHeader, PCIHeaderType},
};

const CONFIG_ADDRESS: u16 = 0xCF8;
const CONFIG_DATA: u16 = 0xCFC;

const CONFIG_ENABLE: u32 = 0x80000000;
pub const BUS_MASTER_ENABLE: u16 = 0x100;
pub const MEMORY_SPACE_ENABLE: u16 = 0x010;
pub const IO_SPACE_ENABLE: u16 = 0x001;

const DEVICE_ID_OFFSET: u8 = 0;
const COMMAND_REGISTER_OFFSET: u8 = 3;

pub static PCI_SUBSYSTEM: PCISubsystem = PCISubsystem::new();

struct PCISubsystemInner {}

impl PCISubsystemInner {
    const fn new() -> Self {
        Self {}
    }

    fn enumerate_pci_devices<F>(&mut self, mut on_each_device: F)
    where
        F: FnMut(PCIDevice),
    {
        for bus in 0..8 {
            for device in 0..32 {
                let device_addr = PCIAddress::device(bus, device);
                if let Some(header) = self.get_pci_header(device_addr) {
                    if !header.is_multifunction {
                        if let Some(device) = self.get_device(header, device_addr) {
                            on_each_device(device);
                        }
                    } else {
                        for function in 0..8 {
                            let addr = PCIAddress::function(bus, device, function);

                            if let Some(header) = self.get_pci_header(addr) {
                                if let Some(device) = self.get_device(header, addr) {
                                    on_each_device(device);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    fn get_device(&mut self, header: PCICommonHeader, addr: PCIAddress) -> Option<PCIDevice> {
        // Only support "General" devices for now
        if header.header_type == PCIHeaderType::General {
            let general_header = unsafe { self.get_pci_general_header(addr) };

            let general_device = PCIGeneralDevice {
                addr,
                common_header: header,
                header: general_header,
                device_class: PCIDeviceClass::from(header.identifiers()),
            };

            Some(PCIDevice::General(general_device))
        } else {
            None
        }
    }

    /// This NEEDS to be called using the address of a device that is in fact a general device
    unsafe fn get_pci_general_header(&mut self, addr: PCIAddress) -> PCIGeneralHeader {
        let mut buffer: [u32; 12] = [0; 12];

        // u32's 4 through 16 are after the common header and make up the general header
        for i in 4..16 {
            buffer[i - 4] = u32::from_le(self.pci_config_read_u32(addr, i as u8));
        }

        PCIGeneralHeader::from(buffer)
    }

    fn get_pci_header(&mut self, addr: PCIAddress) -> Option<PCICommonHeader> {
        // If the PCI bus returns all 1's, then there is no device or function at that address
        if self.pci_config_read_u32(addr, 0) != u32::MAX {
            let mut buffer: [u32; 4] = [0; 4];

            for i in 0..4 {
                buffer[i] = u32::from_le(self.pci_config_read_u32(addr, i as u8));
            }

            let header = PCICommonHeader::from(buffer);
            Some(header)
        } else {
            None
        }
    }

    fn get_pci_config_address(&mut self, addr: PCIAddress, offset: u8) -> u32 {
        let mut address = CONFIG_ENABLE;

        address |= (addr.bus as u32) << 16;
        address |= (addr.device as u32) << 11;
        address |= (addr.function as u32) << 8;
        address |= offset as u32;

        address
    }

    // Offset is in *8 byte words!!!*
    fn pci_config_read_u8(&mut self, addr: PCIAddress, offset: u8) -> u8 {
        // Cut off the last 2 bits
        let u32_offset = offset & 0xFC;
        let address = self.get_pci_config_address(addr, u32_offset);

        unsafe {
            write_io_port_u32(CONFIG_ADDRESS, address.to_le());

            let data = read_io_port_u32(CONFIG_DATA);

            match offset & 0b11 {
                0 => (data & 0xFF) as u8,
                1 => ((data >> 8) & 0xFF) as u8,
                2 => ((data >> 16) & 0xFF) as u8,
                _ => ((data >> 24) & 0xFF) as u8,
            }
        }
    }

    // Offset is in *16 byte words!!!*
    fn pci_config_read_u16(&mut self, addr: PCIAddress, offset: u8) -> u16 {
        // Cut off the last odd bit
        let u32_offset = offset & 0xFE;
        let address = self.get_pci_config_address(addr, u32_offset * 2);

        unsafe {
            write_io_port_u32(CONFIG_ADDRESS, address.to_le());

            let data = read_io_port_u32(CONFIG_DATA);

            // If the offset was odd
            if offset % 2 != 0 {
                (data & 0xFFFF) as u16
            }
            // Of the offset was even
            else {
                (data >> 16) as u16
            }
        }
    }

    // Offset is in *16 byte words!!!*
    fn pci_config_write_u16(&mut self, addr: PCIAddress, offset: u8, value: u16) {
        // Cut off the last odd bit
        let u32_offset = offset & 0xFE;
        let address = self.get_pci_config_address(addr, u32_offset * 2);

        let before = unsafe {
            write_io_port_u32(CONFIG_ADDRESS, address);
            read_io_port_u32(CONFIG_DATA)
        };

        let data_to_write = if offset % 2 != 0 {
            // If the offset was odd
            (value as u32) | (before & 0xFFFF0000)
        } else {
            // Of the offset was even
            ((value as u32) << 16) | (before & 0x0000FFFF)
        };

        unsafe {
            write_io_port_u32(CONFIG_ADDRESS, address.to_le());

            write_io_port_u32(CONFIG_DATA, data_to_write);
        }
    }

    // Offset is in *32 byte words!!!*
    fn pci_config_read_u32(&mut self, addr: PCIAddress, offset: u8) -> u32 {
        let address = self.get_pci_config_address(addr, offset * 4);

        unsafe {
            write_io_port_u32(CONFIG_ADDRESS, address.to_le());

            read_io_port_u32(CONFIG_DATA)
        }
    }

    // Offset is in *32 byte words!!!*
    fn pci_config_write_u32(&mut self, addr: PCIAddress, offset: u8, value: u32) {
        let address = self.get_pci_config_address(addr, offset * 4);

        unsafe {
            write_io_port_u32(CONFIG_ADDRESS, address.to_le());

            write_io_port_u32(CONFIG_DATA, value);
        }
    }
}

pub struct PCISubsystem {
    inner: Mutex<PCISubsystemInner>,
}

impl PCISubsystem {
    pub const fn new() -> Self {
        Self {
            inner: Mutex::new(PCISubsystemInner::new()),
        }
    }

    pub fn enumerate_pci_devices<F>(&self, on_each_device: F)
    where
        F: FnMut(PCIDevice),
    {
        let mut inner = self.inner.lock();
        inner.enumerate_pci_devices(on_each_device)
    }

    pub fn load_pci_drivers(
        &self,
        creators: &[DriverCreator],
        drivers: &mut Vec<Box<dyn PCIDriver>>,
    ) {
        self.enumerate_pci_devices(|device| {
            kprintln!("PCI: Found device {}", device);

            let setup = match self.setup_device(&device) {
                Ok(setup) => setup,
                Err(e) => {
                    kprintln!("PCI: Failed to setup PCI device {}, {:?}", device, e);
                    return;
                }
            };

            for creator in creators {
                match creator {
                    DriverCreator::PCIDriver(creator) => {
                        if creator.detect(&device) {
                            match creator.create(setup) {
                                Ok(driver) => {
                                    kprintln!("Loading driver for PCI device: {}", driver.name());

                                    drivers.push(driver);
                                }
                                Err(e) => {
                                    kprintln!("Failed to load driver: {:?}", e)
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        });
    }

    fn setup_device(&self, device: &PCIDevice) -> Result<PCIDeviceSetup, SystemError> {
        let PCIDevice::General(general_device) = device;

        kprintln!("PCI: Setting up device");

        let irq = if let Some(interrupt_pin) = general_device.interrupt_pin() {
            Some(acpi_pci_get_routing(device.addr(), interrupt_pin)?)
        } else {
            kprintln!("PCI: Device has no interrupt pin");
            None
        };

        Ok(PCIDeviceSetup {
            device: device.clone(),
            address: device.addr(),
            irq,
        })
    }

    pub fn send_command(&self, device: &mut PCIGeneralDevice, command: u16) {
        let mut inner = self.inner.lock();
        let before = inner.pci_config_read_u16(device.addr(), COMMAND_REGISTER_OFFSET);
        inner.pci_config_write_u16(device.addr(), COMMAND_REGISTER_OFFSET, before | command);
    }

    pub fn pci_config_read_u8(&self, address: PCIAddress, offset: u8) -> u8 {
        let mut inner = self.inner.lock();
        inner.pci_config_read_u8(address, offset)
    }

    pub fn pci_config_read_u16(&self, address: PCIAddress, offset: u8) -> u16 {
        let mut inner = self.inner.lock();
        inner.pci_config_read_u16(address, offset)
    }

    pub fn pci_config_read_u32(&self, address: PCIAddress, offset: u8) -> u32 {
        let mut inner = self.inner.lock();
        inner.pci_config_read_u32(address, offset)
    }
}
