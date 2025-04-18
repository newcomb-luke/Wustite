use core::fmt::Display;

#[derive(Debug, Clone, Copy)]
pub struct PCIGeneralHeader {
    pub bar0: u32,
    pub bar1: u32,
    pub bar2: u32,
    pub bar3: u32,
    pub bar4: u32,
    pub bar5: u32,
    pub cardbus_cis_pointer: u32,
    pub subsystem_id: u16,
    pub subsystem_vendor_id: u16,
    pub expansion_rom_base_addr: u32,
    pub capabilities_pointer: u8,
    pub max_latency: u8,
    pub min_grant: u8,
    pub interrupt_pin: Option<InterruptPin>,
    pub interrupt_line: u8,
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
pub struct PCIDeviceIdentifiers {
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
            interrupt_pin: InterruptPin::from_value(((value[11] >> 8) & 0xFF) as u8),
            interrupt_line: (value[11] & 0xFF) as u8,
        }
    }
}

impl PCICommonHeader {
    pub fn identifiers(&self) -> PCIDeviceIdentifiers {
        PCIDeviceIdentifiers {
            class_code: self.class_code,
            subclass: self.subclass,
            prog_if: self.prog_if,
        }
    }
}

pub fn get_vendor_name(vendor_id: u16) -> Option<&'static str> {
    Some(match vendor_id {
        0x8086 => "Intel Corporation",
        0x1B36 => "Red Hat, Inc.",
        0x15AD => "VMWare",
        0x1234 => "QEMU",
        _ => {
            return None;
        }
    })
}

pub fn get_device_name(device_id: u16, vendor_id: u16) -> Option<&'static str> {
    match vendor_id {
        // Intel
        0x8086 => match device_id {
            0x100E => Some("82540EM Gigabit Ethernet Controller"),
            0x10D3 => Some("82574L Gigabit Network Connection"),
            0x1237 => Some("440FX - 82441FX PMC [Natoma]"),
            0x2918 => Some("82801IB (ICH9) LPC Interface Controller"),
            0x2922 => Some("82801IR/IO/IH (ICH9R/DO/DH) 6 port SATA Controller [AHCI mode]"),
            0x2930 => Some("82801I (ICH9 Family) SMBus Controller"),
            0x29C0 => Some("82G33/G31/P35/P31 Express DRAM Controller"),
            0x7000 => Some("82371SB PIIX3 ISA [Natoma/Triton II]"),
            0x7010 => Some("82371SB PIIX3 IDE [Natoma/Triton II]"),
            0x7113 => Some("82371AB/EB/MB PIIX4 ACPI"),
            _ => None,
        },
        // Red Hat, Inc.
        0x1B36 => match device_id {
            0x0010 => Some("QEMU NVM Express"),
            _ => None,
        },
        // VMWare
        0x15AD => match device_id {
            0x0405 => Some("SVGA-II"),
            _ => None,
        },
        // QEMU
        0x1234 => match device_id {
            0x1111 => Some("Standard VGA"),
            _ => None,
        },
        _ => None,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InterruptPin {
    IntA,
    IntB,
    IntC,
    IntD,
}

impl InterruptPin {
    fn from_value(value: u8) -> Option<InterruptPin> {
        match value {
            0 => None,
            1 => Some(Self::IntA),
            2 => Some(Self::IntB),
            3 => Some(Self::IntC),
            4 => Some(Self::IntD),
            _ => None,
        }
    }

    pub fn as_u8(&self) -> u8 {
        match self {
            Self::IntA => 0,
            Self::IntB => 1,
            Self::IntC => 2,
            Self::IntD => 3,
        }
    }
}
