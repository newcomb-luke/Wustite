use crate::kprintln;

use super::{read_io_port_u32, write_io_port_u32};

const CONFIG_ADDRESS: u16 = 0xCF8;
const CONFIG_DATA: u16 = 0xCFC;

const CONFIG_ENABLE: u32 = 0x80000000;

fn pci_config_read_u32(bus: u8, device: u8, function: u8, offset: u8) -> u32 {
    let mut address = CONFIG_ENABLE;

    address |= (bus as u32) << 16;
    address |= (device as u32) << 11;
    address |= (function as u32) << 8;
    address |= (offset as u32) * 4;

    unsafe {
        write_io_port_u32(CONFIG_ADDRESS, address.to_le());

        read_io_port_u32(CONFIG_DATA)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PCICommonHeader {
    pub device_id: u16,
    pub vendor_id: u16,
    pub status: u16,
    pub command: u16,
    pub class_code: u8,
    pub subclass: u8,
    pub prog_if: u8,
    pub revision_id: u8,
    pub bist: u8,
    pub header_type: PCIDeviceType,
    pub latency_timer: u8,
    pub cache_line_size: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PCIDeviceType {
    General,
    PCIToPCIBridge,
    PCIToCardBusBridge,
    Multifunction,
    Unknown,
}

impl From<u8> for PCIDeviceType {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::General,
            1 => Self::PCIToPCIBridge,
            2 => Self::PCIToCardBusBridge,
            o => {
                if (o & (1 << 7)) != 0 {
                    Self::Multifunction
                } else {
                    Self::Unknown
                }
            }
        }
    }
}

impl From<[u32; 4]> for PCICommonHeader {
    fn from(value: [u32; 4]) -> Self {
        let header_type_byte = ((value[3] >> 16) & 0xFF) as u8;

        PCICommonHeader {
            device_id: (value[0] >> 16) as u16,
            vendor_id: (value[0] & 0xFFFF) as u16,
            status: (value[1] >> 16) as u16,
            command: (value[1] & 0xFFFF) as u16,
            class_code: (value[2] >> 24) as u8,
            subclass: ((value[2] >> 16) & 0xFF) as u8,
            prog_if: ((value[2] >> 8) & 0xFF) as u8,
            revision_id: (value[2] & 0xFF) as u8,
            bist: (value[3] >> 24) as u8,
            header_type: header_type_byte.into(),
            latency_timer: ((value[3] >> 8) & 0xFF) as u8,
            cache_line_size: (value[3] & 0xFF) as u8,
        }
    }
}

impl PCICommonHeader {
    pub fn print_summary(&self) {
        kprintln!(
            "    PCI device {:04x}:{:04x}",
            self.vendor_id,
            self.device_id
        );
        kprintln!("         Type: {:?}", self.header_type);
    }
}

pub fn check_pci_device_exists(bus: u8, device: u8) -> Option<PCICommonHeader> {
    // If the PCI bus returns all 1's, then there is no device at that address
    if pci_config_read_u32(bus, device, 0, 0) != u32::MAX {
        let mut buffer: [u32; 4] = [0; 4];

        for i in 0..4 {
            buffer[i] = u32::from_le(pci_config_read_u32(bus, device, 0, i as u8));
        }

        let header = PCICommonHeader::from(buffer);
        Some(header)
    } else {
        None
    }
}
