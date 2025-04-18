use core::fmt::Display;

use super::{
    InterruptPin,
    headers::{
        PCICommonHeader, PCIDeviceClass, PCIGeneralHeader, get_device_name, get_vendor_name,
    },
};

#[derive(Clone, Copy)]
#[non_exhaustive]
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
    pub addr: PCIAddress,
    pub common_header: PCICommonHeader,
    pub header: PCIGeneralHeader,
    pub device_class: PCIDeviceClass,
}

impl Display for PCIGeneralDevice {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!("{} {}: ", self.addr, self.device_class))?;

        let vendor_id = self.common_header.vendor_id;
        let device_id = self.common_header.device_id;

        match (
            get_vendor_name(vendor_id),
            get_device_name(device_id, vendor_id),
        ) {
            (Some(vendor), Some(device)) => f.write_fmt(format_args!("{} {}", vendor, device)),
            (Some(vendor), None) => {
                f.write_fmt(format_args!("{} Unknown ({:04x})", vendor, device_id))
            }
            (None, Some(device)) => {
                // Not sure how this would happen, but what the heck, sure
                f.write_fmt(format_args!("Unknown ({:04x}) {}", vendor_id, device))
            }
            (None, None) => f.write_fmt(format_args!(
                "Unknown ({:04x}:{:04x})",
                vendor_id, device_id
            )),
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

    pub fn interrupt_pin(&self) -> Option<InterruptPin> {
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
    pub fn device(bus: u8, device: u8) -> Self {
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
