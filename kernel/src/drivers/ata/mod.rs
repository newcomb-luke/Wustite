use core::arch::asm;

use spin::Mutex;

use crate::kprintln;

const PRIMARY_IO_BASE: u16 = 0x1F0;
const PRIMARY_CONTROL_BASE: u16 = 0x3F6;
const SECONDARY_IO_BASE: u16 = 0x170;
const SECONDARY_CONTROL_BASE: u16 = 0x376;

pub static PRIMARY_BUS: Bus = Bus::new(PRIMARY_IO_BASE, PRIMARY_CONTROL_BASE);
pub static SECONDARY_BUS: Bus = Bus::new(SECONDARY_IO_BASE, SECONDARY_CONTROL_BASE);

unsafe fn write_io_port_u8(port: u16, data: u8) {
    asm!(
        "mov dx, {:x}",
        "mov al, {}",
        "out dx, al",
        in(reg) port,
        in(reg_byte) data
    );
}

unsafe fn read_io_port_u8(port: u16) -> u8 {
    let data: u8;

    asm!(
        "mov dx, {:x}",
        "in al, dx",
        "mov {}, al",
        in(reg) port,
        lateout(reg_byte) data,
    );

    data
}

unsafe fn write_io_port_u16(port: u16, data: u16) {
    asm!(
        "mov dx, {:x}",
        "mov ax, {:x}",
        "out dx, ax",
        in(reg) port,
        in(reg) data
    );
}

unsafe fn read_io_port_u16(port: u16) -> u16 {
    let data: u16;

    asm!(
        "mov dx, {:x}",
        "in ax, dx",
        "mov {:x}, ax",
        in(reg) port,
        lateout(reg) data,
    );

    data
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Status {
    error: bool,
    index: bool,
    corrected: bool,
    drq: bool,
    srv: bool,
    drive_fault: bool,
    ready: bool,
    busy: bool,
}

impl From<u8> for Status {
    fn from(value: u8) -> Self {
        const ERR: u8 = 1 << 0;
        const IDX: u8 = 1 << 1;
        const CORR: u8 = 1 << 2;
        const DRQ: u8 = 1 << 3;
        const SRV: u8 = 1 << 4;
        const DF: u8 = 1 << 5;
        const RDY: u8 = 1 << 6;
        const BSY: u8 = 1 << 7;

        Self {
            error: value & ERR != 0,
            index: value & IDX != 0,
            corrected: value & CORR != 0,
            drq: value & DRQ != 0,
            srv: value & SRV != 0,
            drive_fault: value & DF != 0,
            ready: value & RDY != 0,
            busy: value & BSY != 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DriveType {
    ATA,
    ATAPI,
    SATA,
    Unknown,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Drive {
    Master,
    Slave,
}

pub struct Bus {
    inner: Mutex<BusInner>,
}

struct BusInner {
    io_port_base: u16,
    control_port_base: u16,
    selected_drive: Option<Drive>,
}

struct IdentifyValue {
    values: [u16; 256],
}

#[derive(Clone, Copy)]
enum BusCommand {
    Identify,
}

impl BusInner {
    unsafe fn write_drive_head(&mut self, data: u8) {
        const DRIVE_HEAD_REGISTER_OFFSET: u16 = 6;
        write_io_port_u8(self.io_port_base + DRIVE_HEAD_REGISTER_OFFSET, data);
    }

    unsafe fn read_regular_status(&mut self) -> u8 {
        const REGULAR_STATUS_OFFSET: u16 = 7;
        read_io_port_u8(self.io_port_base + REGULAR_STATUS_OFFSET)
    }

    unsafe fn read_alternate_status(&mut self) -> u8 {
        const ALTERNATE_STATUS_OFFSET: u16 = 0;
        read_io_port_u8(self.control_port_base + ALTERNATE_STATUS_OFFSET).into()
    }

    unsafe fn write_sector_count(&mut self, count: u8) {
        const SECTOR_COUNT_OFFSET: u16 = 2;
        write_io_port_u8(self.io_port_base + SECTOR_COUNT_OFFSET, count)
    }

    unsafe fn write_lba_lo(&mut self, value: u8) {
        const LBA_LO_OFFSET: u16 = 3;
        write_io_port_u8(self.io_port_base + LBA_LO_OFFSET, value)
    }

    unsafe fn write_lba_mid(&mut self, value: u8) {
        const LBA_MID_OFFSET: u16 = 4;
        write_io_port_u8(self.io_port_base + LBA_MID_OFFSET, value)
    }

    unsafe fn write_lba_hi(&mut self, value: u8) {
        const LBA_HI_OFFSET: u16 = 5;
        write_io_port_u8(self.io_port_base + LBA_HI_OFFSET, value)
    }

    unsafe fn read_sector_count(&mut self) -> u8 {
        const SECTOR_COUNT_OFFSET: u16 = 2;
        read_io_port_u8(self.io_port_base + SECTOR_COUNT_OFFSET)
    }

    unsafe fn read_lba_lo(&mut self) -> u8 {
        const LBA_LO_OFFSET: u16 = 3;
        read_io_port_u8(self.io_port_base + LBA_LO_OFFSET)
    }

    unsafe fn read_lba_mid(&mut self) -> u8 {
        const LBA_MID_OFFSET: u16 = 4;
        read_io_port_u8(self.io_port_base + LBA_MID_OFFSET)
    }

    unsafe fn read_lba_hi(&mut self) -> u8 {
        const LBA_HI_OFFSET: u16 = 5;
        read_io_port_u8(self.io_port_base + LBA_HI_OFFSET)
    }

    unsafe fn write_control_register(&mut self, value: u8) {
        const CONTROL_REGISTER_OFFSET: u16 = 0;
        write_io_port_u8(self.control_port_base + CONTROL_REGISTER_OFFSET, value);
    }

    unsafe fn write_command_register(&mut self, value: u8) {
        const COMMAND_REGISTER_OFFSET: u16 = 7;
        write_io_port_u8(self.io_port_base + COMMAND_REGISTER_OFFSET, value);
    }
}

impl Bus {
    const fn new(io_port_base: u16, control_port_base: u16) -> Self {
        Self {
            inner: Mutex::new(BusInner {
                io_port_base,
                control_port_base,
                selected_drive: None,
            }),
        }
    }

    pub fn disable_interrupts(&self, drive: Drive) {
        self.select_drive(drive);

        {
            let mut bus = self.inner.lock();

            unsafe {
                bus.write_control_register(0b00000010);

                for _ in 0..16 {
                    bus.read_alternate_status();
                }
            }
        }
    }

    pub fn identify(&self, drive: Drive) -> Option<DriveType> {
        self.select_drive(drive);

        {
            let mut bus = self.inner.lock();

            unsafe {
                bus.write_sector_count(0);
                bus.write_lba_lo(0);
                bus.write_lba_mid(0);
                bus.write_lba_hi(0);
            }
        }

        self.send_command(BusCommand::Identify);

        unsafe {
            let mut bus = self.inner.lock();

            for _ in 0..16 {
                bus.read_alternate_status();
            }

            if bus.read_alternate_status() == 0 {
                return None;
            }

            while Status::from(bus.read_alternate_status()).busy {
                x86_64::instructions::nop();
            }

            if Status::from(bus.read_alternate_status()).error {
                // This is not an ATA device

                let mid = bus.read_lba_mid();
                let hi = bus.read_lba_hi();

                if mid == 0x14 && hi == 0xEB {
                    Some(DriveType::ATAPI)
                } else if mid == 0x3C && hi == 0xC3 {
                    Some(DriveType::SATA)
                } else {
                    Some(DriveType::Unknown)
                }
            } else {
                let mid = bus.read_lba_mid();
                let hi = bus.read_lba_hi();

                if mid == 0 && hi == 0 {
                    Some(DriveType::ATA)
                } else {
                    Some(DriveType::Unknown)
                }
            }
        }
    }

    fn send_command(&self, command: BusCommand) {
        let byte = match command {
            BusCommand::Identify => 0xEC,
        };

        {
            let mut bus = self.inner.lock();

            unsafe {
                bus.write_command_register(byte);
            }
        }
    }

    #[inline(never)]
    fn select_drive(&self, drive: Drive) {
        let mut bus = self.inner.lock();

        // If we already have the drive selected, then we don't need to re-select it
        let same = if let Some(current) = bus.selected_drive {
            current == drive
        } else {
            false
        };

        if !same {
            unsafe {
                if drive == Drive::Master {
                    bus.write_drive_head(0xA0);
                } else {
                    bus.write_drive_head(0xB0);
                }

                for _ in 0..16 {
                    bus.read_alternate_status();
                }

                bus.selected_drive = Some(drive);
            }
        }
    }
}
