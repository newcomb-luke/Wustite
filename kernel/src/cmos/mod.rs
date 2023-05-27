use spin::Mutex;

use crate::drivers::{read_io_port_u8, write_io_port_u8};

const CMOS_REGISTER_SELECT_PORT: u16 = 0x70;
const CMOS_IO_PORT: u16 = 0x71;
const CMOS_REGISTER_FLOPPY_DRIVES: u8 = 0x10;

pub static CMOS: CMOSController = CMOSController::new();

struct CMOSInner {
    nmi_disabled: bool,
}

impl CMOSInner {
    fn set_nmi(&mut self, enabled: bool) {
        self.nmi_disabled = !enabled;

        unsafe {
            write_io_port_u8(CMOS_REGISTER_SELECT_PORT, self.nmi_value());
        }
    }

    fn nmi_value(&self) -> u8 {
        if self.nmi_disabled {
            1 << 7
        } else {
            0
        }
    }

    fn read_register(&mut self, register: u8) -> u8 {
        unsafe {
            write_io_port_u8(CMOS_REGISTER_SELECT_PORT, self.nmi_value() | register);
        }

        // Wait a bit
        for _ in 0..10000 {
            x86_64::instructions::nop();
        }

        unsafe { read_io_port_u8(CMOS_IO_PORT) }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FloppyDrive {
    Drive360kb5_25,
    Drive1_2mb5_25,
    Drive720kb3_5,
    Drive1_44mb3_5,
    Drive2_88mb3_5,
    Unknown,
}

impl From<u8> for FloppyDrive {
    fn from(value: u8) -> Self {
        match value {
            1 => FloppyDrive::Drive360kb5_25,
            2 => FloppyDrive::Drive1_2mb5_25,
            3 => FloppyDrive::Drive720kb3_5,
            4 => FloppyDrive::Drive1_44mb3_5,
            5 => FloppyDrive::Drive2_88mb3_5,
            _ => FloppyDrive::Unknown,
        }
    }
}

pub struct CMOSController {
    inner: Mutex<CMOSInner>,
}

impl CMOSController {
    const fn new() -> Self {
        Self {
            inner: Mutex::new(CMOSInner { nmi_disabled: true }),
        }
    }

    pub fn disable_nmi(&self) {
        let mut inner = self.inner.lock();
        inner.set_nmi(true);
    }

    pub fn enable_nmi(&self) {
        let mut inner = self.inner.lock();
        inner.set_nmi(false);
    }

    pub fn get_floppy_drives(&self) -> (Option<FloppyDrive>, Option<FloppyDrive>) {
        let mut inner = self.inner.lock();
        let drives = inner.read_register(CMOS_REGISTER_FLOPPY_DRIVES);
        let high_nibble = drives >> 4;
        let low_nibble = drives & 0x0F;

        let master = if high_nibble != 0 {
            Some(FloppyDrive::from(high_nibble))
        } else {
            None
        };

        let slave = if low_nibble != 0 {
            Some(FloppyDrive::from(low_nibble))
        } else {
            None
        };

        (master, slave)
    }
}
