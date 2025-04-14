use core::fmt::Write;
use spin::Mutex;

use super::{read_io_port_u8, write_io_port_u8};
use crate::logln;

const COM1_PORT: u16 = 0x3F8;

const DATA_REG_OFFSET: u16 = 0;
const INTERRUPT_ENABLE_REG_OFFSET: u16 = 1;
const LINE_CONTROL_REG_OFFSET: u16 = 3;
const DLAB_LEASTSIG_REG_OFFSET: u16 = 0;
const DLAB_MOSTSIG_REG_OFFSET: u16 = 1;
const LINE_STATUS_REG_OFFSET: u16 = 5;

const TRANSMITTER_HOLDING_REGISTER_EMPTY_BIT: u8 = 0b00100000;
const DLAB_BIT: u8 = 0b10000000;

pub static SERIAL0: Mutex<SerialPort> = Mutex::new(SerialPort::new(
    COM1_PORT,
    BaudRate::Baud115200,
    DataBits::Eight,
    StopBits::One,
    Parity::None,
));

pub fn initialize_serial() {
    {
        let mut serial = SERIAL0.lock();
        serial.initialize();
    }

    logln!("[info] Serial initialized");
}

pub struct SerialPort {
    base_port: u16,
    baud_rate: BaudRate,
    data_bits: DataBits,
    stop_bits: StopBits,
    parity: Parity,
}

impl SerialPort {
    const fn new(
        base_port: u16,
        baud_rate: BaudRate,
        data_bits: DataBits,
        stop_bits: StopBits,
        parity: Parity,
    ) -> Self {
        Self {
            base_port,
            baud_rate,
            data_bits,
            stop_bits,
            parity,
        }
    }

    pub fn initialize(&mut self) {
        self.set_divisor_value(self.baud_rate.divisor());

        let line_control_register_value =
            self.data_bits.bits() | self.stop_bits.bits() | self.parity.bits();

        self.write_line_control_reg(line_control_register_value);
    }

    pub fn send_byte(&mut self, byte: u8) {
        self.wait_until_transmission_ready();

        unsafe {
            write_io_port_u8(self.base_port + DATA_REG_OFFSET, byte);
        }
    }

    fn wait_until_transmission_ready(&mut self) {
        loop {
            let status = self.read_line_status_reg();

            if (status & TRANSMITTER_HOLDING_REGISTER_EMPTY_BIT) != 0 {
                break;
            }
        }
    }

    fn set_divisor_value(&mut self, divisor: u16) {
        self.set_dlab_bit();

        let most_significant = (divisor >> 8) as u8;
        let least_significant = (divisor & 0xFF) as u8;

        unsafe {
            write_io_port_u8(self.base_port + DLAB_LEASTSIG_REG_OFFSET, least_significant);
            write_io_port_u8(self.base_port + DLAB_MOSTSIG_REG_OFFSET, most_significant);
        }

        self.clear_dlab_bit();
    }

    #[inline]
    fn read_line_status_reg(&mut self) -> u8 {
        unsafe { read_io_port_u8(self.base_port + LINE_STATUS_REG_OFFSET) }
    }

    #[inline]
    fn read_line_control_reg(&mut self) -> u8 {
        unsafe { read_io_port_u8(self.base_port + LINE_CONTROL_REG_OFFSET) }
    }

    #[inline]
    fn write_line_control_reg(&mut self, value: u8) {
        unsafe { write_io_port_u8(self.base_port + LINE_CONTROL_REG_OFFSET, value) }
    }

    fn set_dlab_bit(&mut self) {
        let before = self.read_line_control_reg();
        self.write_line_control_reg(before | DLAB_BIT);
    }

    fn clear_dlab_bit(&mut self) {
        let before = self.read_line_control_reg();
        self.write_line_control_reg(before & (!DLAB_BIT));
    }
}

impl Write for SerialPort {
    fn write_char(&mut self, c: char) -> core::fmt::Result {
        if c == '\n' {
            self.send_byte(b'\r');
            self.send_byte(b'\n');
        } else {
            self.send_byte(c as u8);
        }
        Ok(())
    }

    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for c in s.chars() {
            self.write_char(c)?;
        }
        Ok(())
    }
}

#[macro_export]
macro_rules! log {
    ($($arg:tt)*) => ($crate::drivers::serial::_log(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! logln {
    () => ($crate::log!("\n"));
    ($($arg:tt)*) => ($crate::log!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _log(args: core::fmt::Arguments) {
    x86_64::instructions::interrupts::without_interrupts(|| {
        let mut port = SERIAL0.lock();
        port.write_fmt(args).unwrap();
    });
}

#[doc(hidden)]
pub fn _log_str(s: &str) {
    x86_64::instructions::interrupts::without_interrupts(|| {
        let mut port = SERIAL0.lock();
        port.write_str(s).unwrap();
    });
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum BaudRate {
    Baud115200,
}

impl BaudRate {
    fn divisor(&self) -> u16 {
        match self {
            Self::Baud115200 => 1,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum DataBits {
    Five,
    Six,
    Seven,
    Eight,
}

impl DataBits {
    fn bits(&self) -> u8 {
        match self {
            Self::Five => 0b00,
            Self::Six => 0b01,
            Self::Seven => 0b10,
            Self::Eight => 0b11,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Parity {
    None,
    Odd,
    Even,
    Mark,
    Space,
}

impl Parity {
    fn bits(&self) -> u8 {
        match self {
            Self::None => 0b000_000,
            Self::Odd => 0b001_000,
            Self::Even => 0b011_000,
            Self::Mark => 0b101_000,
            Self::Space => 0b111_000,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum StopBits {
    One,
    Two,
}

impl StopBits {
    fn bits(&self) -> u8 {
        match self {
            Self::One => 0b0_00,
            Self::Two => 0b1_00,
        }
    }
}
