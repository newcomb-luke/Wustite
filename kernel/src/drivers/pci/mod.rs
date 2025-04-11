use core::fmt::Display;

use alloc::vec::Vec;
use spin::Mutex;

use super::{read_io_port_u32, write_io_port_u32};

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

    fn enumerate_pci_devices(&mut self) -> Vec<PCIDevice> {
        let mut devices = Vec::new();

        for bus in 0..8 {
            for device in 0..32 {
                let device_addr = PCIAddress::new(bus, device);
                if let Some(header) = self.get_pci_header(device_addr) {
                    if !header.is_multifunction {
                        if let Some(device) = self.get_device(header, device_addr) {
                            devices.push(device);
                        }
                    } else {
                        for function in 0..8 {
                            let addr = PCIAddress::function(bus, device, function);

                            if let Some(header) = self.get_pci_header(addr) {
                                if let Some(device) = self.get_device(header, addr) {
                                    devices.push(device);
                                }
                            }
                        }
                    }
                }
            }
        }

        devices
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

    pub fn enumerate_pci_devices(&self) -> Vec<PCIDevice> {
        let mut inner = self.inner.lock();
        inner.enumerate_pci_devices()
    }

    pub fn send_command(&self, device: &mut PCIGeneralDevice, command: u16) {
        let mut inner = self.inner.lock();
        let before = inner.pci_config_read_u16(device.addr(), COMMAND_REGISTER_OFFSET);
        inner.pci_config_write_u16(device.addr(), COMMAND_REGISTER_OFFSET, before | command);
    }
}

#[derive(Clone, Copy)]
pub enum PCIDevice {
    General(PCIGeneralDevice),
}

impl PCIDevice {
    pub fn addr(&self) -> PCIAddress {
        match self {
            Self::General(g) => g.addr,
        }
    }
}

impl Display for PCIDevice {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::General(g) => g.fmt(f),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PCIGeneralDevice {
    addr: PCIAddress,
    common_header: PCICommonHeader,
    header: PCIGeneralHeader,
    device_class: PCIDeviceClass,
}

impl Display for PCIGeneralDevice {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!("{} {}: ", self.addr, self.device_class))?;

        if let Some(device_name) =
            get_device_name(self.common_header.device_id, self.common_header.vendor_id)
        {
            f.write_str(device_name)
        } else {
            f.write_fmt(format_args!(
                "Unknown ({:04x}:{:04x})",
                self.common_header.vendor_id, self.common_header.device_id
            ))
        }
    }
}

impl PCIGeneralDevice {
    pub fn device_class(&self) -> PCIDeviceClass {
        self.device_class
    }

    pub fn vendor_id(&self) -> u16 {
        self.common_header.vendor_id
    }

    pub fn device_id(&self) -> u16 {
        self.common_header.device_id
    }

    pub fn bar0(&self) -> u32 {
        self.header.bar0
    }

    pub fn bar1(&self) -> u32 {
        self.header.bar1
    }

    pub fn bar2(&self) -> u32 {
        self.header.bar2
    }

    pub fn bar3(&self) -> u32 {
        self.header.bar3
    }

    pub fn bar4(&self) -> u32 {
        self.header.bar4
    }

    pub fn prog_if(&self) -> u8 {
        self.common_header.prog_if
    }

    pub fn addr(&self) -> PCIAddress {
        self.addr
    }

    pub fn interrupt_line(&self) -> u8 {
        self.header.interrupt_line
    }

    pub fn interrupt_pin(&self) -> u8 {
        self.header.interrupt_pin
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PCIAddress {
    pub bus: u8,
    pub device: u8,
    pub function: u8,
}

impl PCIAddress {
    pub fn new(bus: u8, device: u8) -> Self {
        Self {
            bus,
            device,
            function: 0,
        }
    }

    pub fn function(bus: u8, device: u8, function: u8) -> Self {
        Self {
            bus,
            device,
            function,
        }
    }
}

impl Display for PCIAddress {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!(
            "{:02x}:{:02x}.{}",
            self.bus, self.device, self.function
        ))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PCIGeneralHeader {
    bar0: u32,
    bar1: u32,
    bar2: u32,
    bar3: u32,
    bar4: u32,
    bar5: u32,
    cardbus_cis_pointer: u32,
    subsystem_id: u16,
    subsystem_vendor_id: u16,
    expansion_rom_base_addr: u32,
    capabilities_pointer: u8,
    max_latency: u8,
    min_grant: u8,
    interrupt_pin: u8,
    interrupt_line: u8,
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

impl Display for Unclassified {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let s = match self {
            Self::NonVGACompatible => "Non-VGA compatible controller",
            Self::VGACompatible => "VGA compatible controller",
            Self::Unknown => "Unknown unclassified",
        };
        f.write_str(s)
    }
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

impl Display for MassStorageController {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let s = match self {
            Self::SCSIController => "SCSI controller",
            Self::IDEController => "IDE controller",
            Self::FloppyController => "Floppy controller",
            Self::IPIController => "IPI controller",
            Self::RAIDController => "RAID controller",
            Self::ATAController => "ATA controller",
            Self::SATAController => "SATA controller",
            Self::SASController => "SAS controller",
            Self::NVMController => "Non-Volatile memory controller",
            Self::Other => "Other storage controller",
        };
        f.write_str(s)
    }
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

impl Display for Bridge {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let s = match self {
            Self::Host => "Host bridge",
            Self::ISA => "ISA bridge",
            Self::EISA => "EISA bridge",
            Self::MCA => "MCA bridge",
            Self::PCIToPCI => "PCI bridge",
            Self::PCMCIA => "PCMCIA bridge",
            Self::NuBus => "NuBus bridge",
            Self::CardBus => "CardBus bridge",
            Self::RACEway => "RACEway bridge",
            Self::InfiniBandToPCI => "InfiniBand bridge",
            Self::Other => "Other bridge",
        };
        f.write_str(s)
    }
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

impl Display for PCIDeviceClass {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let s = match self {
            Self::Unclassified(u) => return u.fmt(f),
            Self::MassStorage(m) => return m.fmt(f),
            Self::Network => "Network controller",
            Self::Display => "Display controller",
            Self::Multimedia => "Multimedia controller",
            Self::Memory => "RAM memory",
            Self::Bridge(b) => return b.fmt(f),
            Self::SimpleCommunication => "Communication controller",
            Self::BaseSystemPeripheral => "System peripheral",
            Self::InputDevice => "Input device",
            Self::DockingStation => "Docking station",
            Self::Processor => "Processor",
            Self::SerialBus => "Serial bus controller",
            Self::Wireless => "Wireless controller",
            Self::Intelligent => "Intelligent controller",
            Self::SatelliteCommunication => "Satellite controller",
            Self::Encryption => "Encryption device",
            Self::SignalProcessing => "Signal processing controller",
            Self::ProcessingAccelerator => "Processing accelerator",
            Self::NonEssential => "Non-essential",
            Self::CoProcessor => "Co-processor",
            Self::Unknown => "Unknown",
        };
        f.write_str(s)
    }
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

impl From<[u32; 12]> for PCIGeneralHeader {
    fn from(value: [u32; 12]) -> Self {
        PCIGeneralHeader {
            bar0: value[0],
            bar1: value[1],
            bar2: value[2],
            bar3: value[3],
            bar4: value[4],
            bar5: value[5],
            cardbus_cis_pointer: value[6],
            subsystem_id: (value[7] >> 16) as u16,
            subsystem_vendor_id: (value[7] & 0xFFFF) as u16,
            expansion_rom_base_addr: value[8],
            capabilities_pointer: (value[9] & 0xFF) as u8,
            // Yes we meant to skip 10, it is just a reserved u32
            max_latency: (value[11] >> 24) as u8,
            min_grant: ((value[11] >> 16) & 0xFF) as u8,
            interrupt_pin: ((value[11] >> 8) & 0xFF) as u8,
            interrupt_line: (value[11] & 0xFF) as u8,
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
}

fn get_device_name(device_id: u16, vendor_id: u16) -> Option<&'static str> {
    match vendor_id {
        // Intel
        0x8086 => match device_id {
            0x7010 => Some("Intel PIIX3 IDE"),
            _ => None,
        },
        // Red Hat, Inc.
        0x1B36 => match device_id {
            0x0010 => Some("Red Hat QEMU NVM Express"),
            _ => None,
        },
        // VMWare
        0x15AD => match device_id {
            0x0405 => Some("VMWare SVGA-II"),
            _ => None,
        },
        _ => None,
    }
}
