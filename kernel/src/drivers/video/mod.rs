use core::fmt::{Result, Write};
use core::{arch::asm, cell::RefCell};

use lazy_static::lazy_static;
use spin::Mutex;
use x86_64::instructions::port::Port;

const VIDEO_MEMORY: *mut u16 = 0xB8000 as *mut u16;
const NUM_COLUMNS: usize = 80;
const NUM_ROWS: usize = 25;

const VGA_ADDR_PORT: Port<u8> = Port::new(0x03D4);
const VGA_DATA_PORT: Port<u8> = Port::new(0x03D5);

const CURSOR_LOC_LOW: VGARegister = VGARegister::new(0x0f);
const CURSOR_LOC_HIGH: VGARegister = VGARegister::new(0x0e);

const REGULAR_FG: Color = Color::White;
const ERROR_FG: Color = Color::LightRed;

lazy_static! {
    pub static ref TEXT_BUFFER: Mutex<TextBuffer> =
        Mutex::new(TextBuffer::new(REGULAR_FG, Color::Black));
}

#[macro_export]
macro_rules! kprint {
    ($($arg:tt)*) => ($crate::drivers::video::_kprint(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! kprintln {
    () => ($crate::kprint!("\n"));
    ($($arg:tt)*) => ($crate::kprint!("{}\n", format_args!($($arg)*)));
}

#[macro_export]
macro_rules! keprint {
    ($($arg:tt)*) => ($crate::drivers::video::_keprint(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! keprintln {
    () => ($crate::keprint!("\n"));
    ($($arg:tt)*) => ($crate::keprint!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _kprint(args: core::fmt::Arguments) {
    use core::fmt::Write;
    TEXT_BUFFER.lock().write_fmt(args).unwrap();
}

#[doc(hidden)]
pub fn _keprint(args: core::fmt::Arguments) {
    use core::fmt::Write;
    TEXT_BUFFER.lock().set_fg(ERROR_FG);
    TEXT_BUFFER.lock().write_fmt(args).unwrap();
    TEXT_BUFFER.lock().set_fg(REGULAR_FG);
}

#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
#[allow(dead_code)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    LightMagenta = 13,
    Yellow = 14,
    White = 15,
}

pub struct VGARegister(u8);

impl VGARegister {
    pub const fn new(reg: u8) -> Self {
        Self(reg)
    }

    pub unsafe fn write(&self, value: u8) {
        VGA_ADDR_PORT.write(self.0);
        VGA_DATA_PORT.write(value);
    }
}

pub struct TextBuffer {
    line: usize,
    col: usize,
    fg: Color,
    bg: Color,
}

impl TextBuffer {
    pub fn new(fg: Color, bg: Color) -> Self {
        let n = Self {
            line: 0,
            col: 0,
            fg,
            bg,
        };

        n.clear_screen();

        n
    }

    pub fn put_char(&mut self, c: char) {
        if c == '\n' {
            self.line += 1;
            self.col = 0;
            self.set_cursor(0, self.line);
        } else if c == '\r' {
            self.col = 0;
            self.set_cursor(0, self.line);
        } else {
            let val = self.value_from_char(c);
            let offset = (self.col + NUM_COLUMNS * self.line) as isize;

            unsafe {
                *VIDEO_MEMORY.offset(offset) = val;
            }

            self.increment_pos();

            self.set_cursor(self.col, self.line);
        }
    }

    pub fn put_str(&mut self, s: &str) {
        let bytes = s.as_bytes();

        for b in bytes {
            self.put_char(*b as char);
        }
    }

    fn increment_pos(&mut self) {
        self.col += 1;

        if self.col > NUM_COLUMNS {
            self.line += 1;
            self.col = 0;
        }
    }

    pub fn clear_screen(&self) {
        let total_size = NUM_COLUMNS * NUM_ROWS;

        for i in (0..total_size).map(|i| i as isize) {
            unsafe {
                *VIDEO_MEMORY.offset(i) = self.value_from_char(' ');
            }
        }

        self.set_cursor(0, 0);
    }

    fn value_from_char(&self, c: char) -> u16 {
        let mut value: u16 = 0;
        value += (self.bg as u16) << 12;
        value += (self.fg as u16) << 8;
        match c as u8 {
            0x20..=0x7e | b'\n' | b'\r' => {
                value += (c as u8) as u16;
            }
            _ => {
                value += 0xfe;
            }
        }

        value
    }

    fn set_cursor(&self, col: usize, row: usize) {
        let offset = (row * NUM_COLUMNS + col) as u16;
        let low = (offset & 0xFF) as u8;
        let high = (offset >> 8) as u8;

        unsafe {
            CURSOR_LOC_LOW.write(low);
            CURSOR_LOC_HIGH.write(high);
        }
    }

    pub fn set_fg(&mut self, fg: Color) {
        self.fg = fg;
    }

    pub fn set_bg(&mut self, bg: Color) {
        self.bg = bg;
    }
}

impl Write for TextBuffer {
    fn write_str(&mut self, s: &str) -> Result {
        self.put_str(s);
        Ok(())
    }
    fn write_char(&mut self, c: char) -> Result {
        self.put_char(c);
        Ok(())
    }
}
