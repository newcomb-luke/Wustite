#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AcpiDevice {
    PS2Mouse,
    PS2Keyboard,
    COMPort16550ACompatible,
    StandardFloppyDiskController,
    StandardLPTParallelPort,
    AtRealTimeClock,
    PCIInterruptLinkDevice,
    HPETSystemTimer,
    PCIBus,
}

pub fn acpi_device_from_hid(hid: u64) -> Option<AcpiDevice> {
    Some(match hid {
        0x00000000130fd041 => AcpiDevice::PS2Mouse,
        0x000000000303d041 => AcpiDevice::PS2Keyboard,
        0x000000000105d041 => AcpiDevice::COMPort16550ACompatible,
        0x000000000007d041 => AcpiDevice::StandardFloppyDiskController,
        0x000000000004d041 => AcpiDevice::StandardLPTParallelPort,
        0x00000000000bd041 => AcpiDevice::AtRealTimeClock,
        0x000000000f0cd041 => AcpiDevice::PCIInterruptLinkDevice,
        0x000000000301d041 => AcpiDevice::HPETSystemTimer,
        0x00000000030ad041 => AcpiDevice::PCIBus,
        _ => {
            return None;
        }
    })
}
