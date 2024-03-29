#![allow(dead_code)]

use core::fmt::{Result, Write};

use lazy_static::lazy_static;
use spin::Mutex;

use crate::drivers::write_io_port_u8;

const VIDEO_MEMORY: *mut u16 = 0xB8000 as *mut u16;
const NUM_COLUMNS: usize = 80;
const NUM_ROWS: usize = 25;

const CURSOR_LOC_LOW: VGARegister = VGARegister::new(0x0f);
const CURSOR_LOC_HIGH: VGARegister = VGARegister::new(0x0e);

const REGULAR_FG: Color = Color::White;
const ERROR_FG: Color = Color::LightRed;

lazy_static! {
    pub static ref TEXT_BUFFER: Mutex<TextBuffer> =
        Mutex::new(TextBuffer::new(REGULAR_FG, Color::Black));
}

#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
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
        const VGA_ADDR_PORT: u16 = 0x03D4;
        const VGA_DATA_PORT: u16 = 0x03D5;

        write_io_port_u8(VGA_ADDR_PORT, self.0);
        write_io_port_u8(VGA_DATA_PORT, value);
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
        let mut n = Self {
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

            if self.line >= NUM_ROWS {
                self.shift_screen_up();
            }

            self.set_cursor(self.col, self.line);
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

    fn shift_screen_up(&mut self) {
        for row in 1..NUM_ROWS {
            for col in 0..NUM_COLUMNS {
                let offset = (col + NUM_COLUMNS * row) as isize;
                let previous = (col + NUM_COLUMNS * (row - 1)) as isize;

                unsafe {
                    VIDEO_MEMORY
                        .offset(previous)
                        .write_volatile((VIDEO_MEMORY).offset(offset).read_volatile());
                }
            }
        }

        self.line = NUM_ROWS - 1;
        self.clear_line(self.line);
    }

    fn clear_line(&self, line: usize) {
        let offset = (self.col + NUM_COLUMNS * line) as isize;

        for i in 0..(NUM_COLUMNS as isize) {
            unsafe {
                *VIDEO_MEMORY.offset(offset + i) = self.value_from_char(' ');
            }
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

            if self.line >= NUM_ROWS {
                self.shift_screen_up();
            }
        }
    }

    pub fn clear_screen(&mut self) {
        let total_size = NUM_COLUMNS * NUM_ROWS;

        for i in (0..total_size).map(|i| i as isize) {
            unsafe {
                VIDEO_MEMORY
                    .offset(i)
                    .write_volatile(self.value_from_char(' '));
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

    fn set_cursor(&mut self, col: usize, row: usize) {
        let offset = (row * NUM_COLUMNS + col) as u16;
        let low = (offset & 0xFF) as u8;
        let high = (offset >> 8) as u8;

        unsafe {
            CURSOR_LOC_LOW.write(low);
            CURSOR_LOC_HIGH.write(high);
        }

        self.line = row;
        self.col = col;
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

macro_rules! print {
    ($($arg:tt)*) => ($crate::drivers::video::vga::text::_print(format_args!($($arg)*)));
}

macro_rules! println {
    () => ($crate::drivers::video::vga::text::print!("\n"));
    ($($arg:tt)*) => ($crate::drivers::video::vga::text::print!("{}\n", format_args!($($arg)*)));
}

macro_rules! eprint {
    ($($arg:tt)*) => ($crate::drivers::video::vga::text::_eprint(format_args!($($arg)*)));
}

macro_rules! eprintln {
    () => ($crate::drivers::video::vga::text::eprint!("\n"));
    ($($arg:tt)*) => ($crate::drivers::video::vga::text::eprint!("{}\n", format_args!($($arg)*)));
}

pub(crate) use eprint;
pub(crate) use eprintln;
pub(crate) use print;
pub(crate) use println;

#[doc(hidden)]
pub fn _print(args: core::fmt::Arguments) {
    x86_64::instructions::interrupts::without_interrupts(|| {
        let mut text_buffer = TEXT_BUFFER.lock();
        text_buffer.write_fmt(args).unwrap();
    });
}

#[doc(hidden)]
pub fn _eprint(args: core::fmt::Arguments) {
    x86_64::instructions::interrupts::without_interrupts(|| {
        let mut text_buffer = TEXT_BUFFER.lock();
        text_buffer.set_fg(Color::Red);
        text_buffer.write_fmt(args).unwrap();
        text_buffer.set_fg(Color::White);
    });
}
