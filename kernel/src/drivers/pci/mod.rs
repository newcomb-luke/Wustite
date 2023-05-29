use crate::{print, println};

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
    pub header_type: PCIHeaderType,
    pub latency_timer: u8,
    pub cache_line_size: u8,
    pub is_multifunction: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PCIHeaderType {
    General,
    PCIToPCIBridge,
    PCIToCardBusBridge,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Unclassified {
    NonVGACompatible,
    VGACompatible,
    Unknown,
}

impl From<PCIDeviceIdentifiers> for Unclassified {
    fn from(value: PCIDeviceIdentifiers) -> Self {
        match value.subclass {
            0 => Self::NonVGACompatible,
            1 => Self::VGACompatible,
            _ => Self::Unknown,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MassStorageController {
    SCSIController,
    IDEController,
    FloppyController,
    IPIController,
    RAIDController,
    ATAController,
    SATAController,
    SASController,
    NVMController,
    Other,
}

impl From<PCIDeviceIdentifiers> for MassStorageController {
    fn from(value: PCIDeviceIdentifiers) -> Self {
        match value.subclass {
            0 => Self::SCSIController,
            1 => Self::IDEController,
            2 => Self::FloppyController,
            3 => Self::IPIController,
            4 => Self::RAIDController,
            5 => Self::ATAController,
            6 => Self::SATAController,
            7 => Self::SASController,
            8 => Self::NVMController,
            _ => Self::Other,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Bridge {
    Host,
    ISA,
    EISA,
    MCA,
    PCIToPCI,
    PCMCIA,
    NuBus,
    CardBus,
    RACEway,
    InfiniBandToPCI,
    Other,
}

impl From<PCIDeviceIdentifiers> for Bridge {
    fn from(value: PCIDeviceIdentifiers) -> Self {
        match value.subclass {
            0 => Self::Host,
            1 => Self::ISA,
            2 => Self::EISA,
            3 => Self::MCA,
            4 | 9 => Self::PCIToPCI,
            5 => Self::PCMCIA,
            6 => Self::NuBus,
            7 => Self::CardBus,
            8 => Self::RACEway,
            10 => Self::InfiniBandToPCI,
            _ => Self::Other,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct PCIDeviceIdentifiers {
    class_code: u8,
    subclass: u8,
    prog_if: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PCIDeviceClass {
    Unclassified(Unclassified),
    MassStorage(MassStorageController),
    Network,
    Display,
    Multimedia,
    Memory,
    Bridge(Bridge),
    SimpleCommunication,
    BaseSystemPeripheral,
    InputDevice,
    DockingStation,
    Processor,
    SerialBus,
    Wireless,
    Intelligent,
    SatelliteCommunication,
    Encryption,
    SignalProcessing,
    ProcessingAccelerator,
    NonEssential,
    CoProcessor,
    Unknown,
}

impl From<PCIDeviceIdentifiers> for PCIDeviceClass {
    fn from(value: PCIDeviceIdentifiers) -> Self {
        match value.class_code {
            0 => Self::Unclassified(Unclassified::from(value)),
            1 => Self::MassStorage(MassStorageController::from(value)),
            2 => Self::Network,
            3 => Self::Display,
            4 => Self::Multimedia,
            5 => Self::Memory,
            6 => Self::Bridge(Bridge::from(value)),
            7 => Self::SimpleCommunication,
            8 => Self::BaseSystemPeripheral,
            9 => Self::InputDevice,
            10 => Self::DockingStation,
            11 => Self::Processor,
            12 => Self::SerialBus,
            13 => Self::Wireless,
            14 => Self::Intelligent,
            15 => Self::SatelliteCommunication,
            16 => Self::Encryption,
            17 => Self::SignalProcessing,
            18 => Self::ProcessingAccelerator,
            19 => Self::NonEssential,
            0x40 => Self::CoProcessor,
            _ => Self::Unknown,
        }
    }
}

impl From<u8> for PCIHeaderType {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::General,
            1 => Self::PCIToPCIBridge,
            2 => Self::PCIToCardBusBridge,
            _ => Self::Unknown,
        }
    }
}

impl From<[u32; 4]> for PCICommonHeader {
    fn from(value: [u32; 4]) -> Self {
        let mut header_type_byte = ((value[3] >> 16) & 0xFF) as u8;

        let is_multifunction = (header_type_byte & (1 << 7)) != 0;

        if is_multifunction {
            header_type_byte &= !(1 << 7);
        }

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
            is_multifunction,
        }
    }
}

impl PCICommonHeader {
    fn identifiers(&self) -> PCIDeviceIdentifiers {
        PCIDeviceIdentifiers {
            class_code: self.class_code,
            subclass: self.subclass,
            prog_if: self.prog_if,
        }
    }

    pub fn print_summary(&self, function: bool) {
        if function {
            print!("    ");
        }

        println!(
            "    PCI device {:04x}:{:04x}",
            self.vendor_id, self.device_id
        );
        if function {
            print!("    ");
        }
        println!("         Type: {:?}", self.header_type);
        if function {
            print!("    ");
        }
        println!(
            "         Function: {:?}",
            PCIDeviceClass::from(self.identifiers())
        );
    }
}

fn get_pci_header(bus: u8, device: u8, function: u8) -> Option<PCICommonHeader> {
    // If the PCI bus returns all 1's, then there is no device or function at that address
    if pci_config_read_u32(bus, device, function, 0) != u32::MAX {
        let mut buffer: [u32; 4] = [0; 4];

        for i in 0..4 {
            buffer[i] = u32::from_le(pci_config_read_u32(bus, device, function, i as u8));
        }

        let header = PCICommonHeader::from(buffer);
        Some(header)
    } else {
        None
    }
}

pub fn check_pci_device_exists(bus: u8, device: u8) -> Option<PCICommonHeader> {
    get_pci_header(bus, device, 0)
}

/// Make sure to only call this function with devices that report being Multifunction,
/// otherwise you might get 256 of the same function 0
pub fn check_pci_device_function_exists(
    bus: u8,
    device: u8,
    function: u8,
) -> Option<PCICommonHeader> {
    get_pci_header(bus, device, function)
}
