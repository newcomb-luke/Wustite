use core::arch::asm;

const VIDEO_MEMORY: *mut u16 = 0xb8000 as *mut u16;
const VGA_ADDR_REGISTER: u16 = 0x03d4;
const VGA_DATA_REGISTER: u16 = 0x03d5;

#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum BgColor {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
}

#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum FgColor {
    DarkGrey = 0,
    LightBlue = 1,
    LightGreen = 2,
    LightCyan = 3,
    LightRed = 4,
    LightMagenta = 5,
    Yellow = 6,
    White = 7,
}

pub struct TextBuffer {
    line: usize,
    col: usize,
    max_line: usize,
    max_col: usize,
    fg: FgColor,
    bg: BgColor,
    bright: bool,
}

impl TextBuffer {
    pub fn new(fg: FgColor, bright: bool, bg: BgColor) -> Self {
        let n = Self {
            line: 0,
            col: 0,
            max_col: 80,
            max_line: 25,
            fg,
            bg,
            bright,
        };

        n.clear_screen();

        n
    }

    pub fn putc(&mut self, c: char) {
        if c != '\n' {
            let val = self.value_from_char(c);
            let offset = (self.col + self.max_col * self.line) as isize;

            unsafe {
                *VIDEO_MEMORY.offset(offset) = val;
            }

            self.increment_pos();

            self.set_cursor(self.col, self.line);
        } else {
            self.line += 1;
            self.col = 0;
            self.set_cursor(0, self.line);
        }
    }

    pub fn puts(&mut self, s: &str) {
        let bytes = s.as_bytes();

        for b in bytes {
            self.putc(*b as char);
        }
    }

    pub fn putln(&mut self, s: &str) {
        self.puts(s);
        self.putc('\n');
    }

    fn increment_pos(&mut self) {
        self.col += 1;

        if self.col > self.max_col {
            self.line += 1;
            self.col = 0;
        }
    }

    fn clear_screen(&self) {
        let total_size = self.max_col * self.max_line;

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
        value += (self.fg as u16 | if self.bright { 8 } else { 0 }) << 8;
        value += c as u8 as u16;

        value
    }

    fn write_vga_register(reg: u8, data: u8) {
        unsafe {
            asm!(
            "mov dx, {0:x}", // mov dx, VGA_ADDR_REGISTER
            "mov al, {1}", // mov al, reg
            "out dx, al", // out dx, al
            "mov dx, {2:x}", // mov dx, VGA_DATA_REGISTER
            "mov al, {3}", // mov al, data
            "out dx, al", // out dx, al
            in(reg_abcd) VGA_ADDR_REGISTER,
            in(reg_byte) reg,
            in(reg_abcd) VGA_DATA_REGISTER,
            in(reg_byte) data,
            in("al") 0u8,
            in("dx") 0u16,
            );
        }
    }

    fn set_cursor(&self, col: usize, row: usize) {
        const CURSOR_LOC_LOW: u8 = 0x0f;
        const CURSOR_LOC_HIGH: u8 = 0x0e;

        let offset = (row * self.max_col + col) as u16;
        let low = (offset & 0xFF) as u8;
        let high = (offset >> 8) as u8;

        Self::write_vga_register(CURSOR_LOC_LOW, low);
        Self::write_vga_register(CURSOR_LOC_HIGH, high);
    }
}
