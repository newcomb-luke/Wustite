use core::fmt::Write;

use spin::Mutex;

use crate::drivers::{read_io_port_u8, write_io_port_u8};

use self::font::get_char;

mod font;

// http://www.osdever.net/FreeVGA/vga/graphreg.htm#05
//
// http://computer-programming-forum.com/45-asm/5a88e2d82140a8ef.htm

const MAIN_INDEX_REGISTER_PORT: u16 = 0x3C0;
const MAIN_INDEX_REGISTER_RESET_PORT: u16 = 0x3DA;
const MISC_OUTPUT_REGISTER_WRITE_PORT: u16 = 0x3C2;
const MISC_OUTPUT_REGISTER_READ_PORT: u16 = 0x3C2;
const SEQUENCER_REGISTER_SELECT_PORT: u16 = 0x3C4;
const SEQUENCER_REGISTER_DATA_PORT: u16 = 0x3C5;
const GRAPHICS_REGISTER_SELECT_PORT: u16 = 0x3CE;
const GRAPHICS_REGISTER_DATA_PORT: u16 = 0x3CF;
const CRTC_REGISTER_SELECT_PORT: u16 = 0x3D4;
const CRTC_REGISTER_DATA_PORT: u16 = 0x3D5;

const VIDEO_MEMORY: *mut u8 = 0xA0000 as *mut u8;
const VIDEO_MEMORY_U32: *mut u32 = 0xA0000 as *mut u32;

const RED_PLANE: u8 = 0b0100;
const GREEN_PLANE: u8 = 0b0010;
const BLUE_PLANE: u8 = 0b0001;
const HIGHLIGHT_PLANE: u8 = 0b1000;

const HEIGHT: usize = 480;
const WIDTH: usize = 640;
const WIDTH_U32S: usize = 20;
const WIDTH_U8S: usize = 80;
const MEM_SIZE_U32S: usize = HEIGHT * WIDTH_U32S;

pub static GRAPHICS: VGAGraphics = VGAGraphics::new();
pub static TEXT_BUFFER: Mutex<VGATextBuffer> = Mutex::new(VGATextBuffer::new(0, 0));

pub struct VGATextBuffer {
    x: usize,
    y: usize,
    length: usize,
    row: usize,
}

impl VGATextBuffer {
    pub const fn new(x: usize, y: usize) -> Self {
        Self {
            x,
            y,
            length: 0,
            row: y,
        }
    }

    pub fn backspace(&mut self) {
        if self.length != 0 {
            self.length -= 1;

            GRAPHICS.set_color(CharColor::White);
            GRAPHICS.draw_char(' ', self.x + self.length, self.row);
        } else if self.row > self.y {
            self.length = WIDTH_U8S;
            self.row -= 9;
        }
    }

    pub fn newline(&mut self) {
        self.length = 0;
        self.row += 9;
    }

    pub fn append_str(&mut self, s: &str) {
        for c in s.chars() {
            self.append_char(c);
        }
    }

    pub fn append_char(&mut self, mut c: char) {
        if c == '\n' {
            self.newline();
        } else {
            if c.is_lowercase() {
                c = c.to_uppercase().next().unwrap();
            }

            if self.length >= WIDTH_U8S {
                self.newline();
            }

            GRAPHICS.draw_char(c, self.x + self.length, self.row);

            self.length += 1;
        }
    }

    pub fn with_color<F>(&mut self, color: CharColor, mut f: F)
    where
        F: FnMut(&mut Self),
    {
        let before = GRAPHICS.get_color();
        GRAPHICS.set_color(color);

        f(self);

        GRAPHICS.set_color(before);
    }

    pub fn append_str_colored(&mut self, s: &str, color: CharColor) {
        let before = GRAPHICS.get_color();
        GRAPHICS.set_color(color);

        for c in s.chars() {
            self.append_char(c);
        }

        GRAPHICS.set_color(before);
    }
}

impl Write for VGATextBuffer {
    fn write_char(&mut self, c: char) -> core::fmt::Result {
        self.append_char(c);
        Ok(())
    }

    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.append_str(s);
        Ok(())
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::drivers::video::vga::graphics::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[macro_export]
macro_rules! eprint {
    ($($arg:tt)*) => ($crate::drivers::video::vga::graphics::_eprint(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! eprintln {
    () => ($crate::eprint!("\n"));
    ($($arg:tt)*) => ($crate::eprint!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: core::fmt::Arguments) {
    x86_64::instructions::interrupts::without_interrupts(|| {
        let mut buffer = TEXT_BUFFER.lock();
        buffer.write_fmt(args).unwrap()
    });
}

#[doc(hidden)]
pub fn _eprint(args: core::fmt::Arguments) {
    x86_64::instructions::interrupts::without_interrupts(|| {
        let mut buffer = TEXT_BUFFER.lock();
        buffer.with_color(CharColor::Red, |b| b.write_fmt(args).unwrap());
    });
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CharColor {
    Red,
    Green,
    Blue,
    White,
}

struct VGAGraphicsInner {
    color: CharColor,
}

impl VGAGraphicsInner {
    const fn new() -> Self {
        Self {
            color: CharColor::White,
        }
    }

    fn init(&mut self) {
        switch_to_graphics_mode();
    }

    fn fill_screen(&mut self, color: u8) {
        unsafe {
            set_write_memory_planes(color);
            for offset in 0..MEM_SIZE_U32S {
                VIDEO_MEMORY_U32.add(offset).write_volatile(u32::MAX);
            }
        }
    }

    fn clear_screen(&mut self) {
        unsafe {
            set_write_memory_planes(0b1111);
            // It only takes 20 u32's to fill one line of the screen
            for offset in 0..MEM_SIZE_U32S {
                VIDEO_MEMORY_U32.add(offset).write_volatile(0);
            }
        }
    }

    fn set_color(&mut self, color: CharColor) {
        self.color = color;

        unsafe {
            let planes = match color {
                CharColor::Red => 0b1100,
                CharColor::Green => 0b1010,
                CharColor::Blue => 0b1001,
                CharColor::White => 0b1111,
            };

            set_write_memory_planes(planes);
        }
    }

    fn get_color(&mut self) -> CharColor {
        self.color
    }

    fn draw_char(&mut self, c: char, x: usize, y: usize) {
        unsafe {
            if let Some(bitmap) = get_char(c) {
                for (i, row) in bitmap.iter().enumerate() {
                    VIDEO_MEMORY
                        .add((i + y) * WIDTH_U8S + x)
                        .write_volatile(*row);
                }
            }
        }
    }
}

pub struct VGAGraphics {
    inner: Mutex<VGAGraphicsInner>,
}

impl VGAGraphics {
    const fn new() -> Self {
        Self {
            inner: Mutex::new(VGAGraphicsInner::new()),
        }
    }

    pub fn init(&self) {
        let mut inner = self.inner.lock();
        inner.init();
    }

    pub fn fill_screen(&self, color: u8) {
        let mut inner = self.inner.lock();
        inner.fill_screen(color);
    }

    pub fn clear_screen(&self) {
        let mut inner = self.inner.lock();
        inner.clear_screen();
    }

    pub fn set_color(&self, color: CharColor) {
        let mut inner = self.inner.lock();
        inner.set_color(color);
    }

    pub fn get_color(&self) -> CharColor {
        let mut inner = self.inner.lock();
        inner.get_color()
    }

    pub fn draw_char(&self, c: char, x: usize, y: usize) {
        let mut inner = self.inner.lock();
        inner.draw_char(c, x, y);
    }

    pub fn draw_str(&self, s: &str, x: usize, y: usize) {
        let mut inner = self.inner.lock();

        for (i, c) in s.chars().enumerate() {
            inner.draw_char(c, x + i, y);
        }
    }
}

/// Enables VGA 640x480 16 bit color mode
fn switch_to_graphics_mode() {
    unsafe {
        enable_crtc_register();

        unlock_crtc();

        disable_display();

        // Mode control register
        write_main_register(0x10, 0x01);
        // Overscan register
        write_main_register(0x11, 0x00);
        // Color plane enable register
        write_main_register(0x12, 0x0F);
        // Horizontal panning register
        write_main_register(0x13, 0x00);
        // Color select register
        write_main_register(0x14, 0x00);

        write_io_port_u8(MISC_OUTPUT_REGISTER_WRITE_PORT, 0xE3);

        // Clock mode register
        write_sequencer_register(0x01, 0x01);
        // Character select register
        write_sequencer_register(0x03, 0x00);
        // Memory mode register
        write_sequencer_register(0x04, 0x02);
        // Mode register
        write_graphics_register(0x05, 0x00);
        // Misc register
        write_graphics_register(0x06, 0x05);
        // Horizontal total register
        write_crtc_register(0x00, 0x5F);
        // Horizontal display enable end register
        write_crtc_register(0x01, 0x4F);
        // Horizontal blank start register
        write_crtc_register(0x02, 0x50);
        // Horizontal blank end register
        write_crtc_register(0x03, 0x82);
        // Horizontal retrace start register
        write_crtc_register(0x04, 0x54);
        // Horizontal retrace end register
        write_crtc_register(0x05, 0x80);
        // Vertical total register
        write_crtc_register(0x06, 0x0B);
        // Overflow register
        write_crtc_register(0x07, 0x3E);
        // Preset row scan register
        write_crtc_register(0x08, 0x00);
        // Maximum scan line register
        write_crtc_register(0x09, 0x40);
        // Vertical retrace start register
        write_crtc_register(0x10, 0xEA);
        // Vertical retrace end register
        write_crtc_register(0x11, 0x8C);
        // Vertical display enable end register
        write_crtc_register(0x12, 0xDF);
        // Logical width register
        write_crtc_register(0x13, 0x28);
        // Underline location register
        write_crtc_register(0x14, 0x00);
        // Vertical blank start register
        write_crtc_register(0x15, 0xE7);
        // Vertical blank end register
        write_crtc_register(0x16, 0x04);
        // Mode control register
        write_crtc_register(0x17, 0xE3);

        enable_display();
    }
}

unsafe fn set_write_memory_planes(planes: u8) {
    write_sequencer_register(0x02, planes);
}

unsafe fn draw_square() {
    set_write_memory_planes(GREEN_PLANE);

    let h_start = 64;
    let w_start = 8;
    let height = 64;
    let width = 8;

    for i in h_start..(h_start + height) {
        for j in w_start..(w_start + width) {
            VIDEO_MEMORY.add(i * 80 + j).write_volatile(0xFF);
        }
    }
}

unsafe fn write_blue() {
    const VIDEO_MEMORY: *mut u8 = 0xA0000 as *mut u8;

    set_write_memory_planes(0b1110);

    for i in 0..(64 * 1024) {
        VIDEO_MEMORY.add(i).write_volatile(0x00);
    }

    set_write_memory_planes(0b0001);

    for i in 0..(64 * 1024) {
        VIDEO_MEMORY.add(i).write_volatile(0xFF);
    }
}

unsafe fn write_main_register(index: u8, value: u8) {
    // Reset the port to be in the index state
    read_io_port_u8(MAIN_INDEX_REGISTER_RESET_PORT);

    // Write the register we want to write to
    write_io_port_u8(MAIN_INDEX_REGISTER_PORT, index);
    // Write the data we want to write to that register
    write_io_port_u8(MAIN_INDEX_REGISTER_PORT, value);
}

unsafe fn read_main_register(index: u8) -> u8 {
    // Reset the port to be in the index state
    read_io_port_u8(MAIN_INDEX_REGISTER_RESET_PORT);

    // Write the register we want to read from
    write_io_port_u8(MAIN_INDEX_REGISTER_PORT, index);
    // Read the data we want from that register
    read_io_port_u8(MAIN_INDEX_REGISTER_PORT)
}

unsafe fn write_sequencer_register(index: u8, value: u8) {
    write_io_port_u8(SEQUENCER_REGISTER_SELECT_PORT, index);
    write_io_port_u8(SEQUENCER_REGISTER_DATA_PORT, value);
}

unsafe fn read_sequencer_register(index: u8) -> u8 {
    write_io_port_u8(SEQUENCER_REGISTER_SELECT_PORT, index);
    read_io_port_u8(SEQUENCER_REGISTER_DATA_PORT)
}

unsafe fn write_graphics_register(index: u8, value: u8) {
    write_io_port_u8(GRAPHICS_REGISTER_SELECT_PORT, index);
    write_io_port_u8(GRAPHICS_REGISTER_DATA_PORT, value);
}

unsafe fn read_graphics_register(index: u8) -> u8 {
    write_io_port_u8(GRAPHICS_REGISTER_SELECT_PORT, index);
    read_io_port_u8(GRAPHICS_REGISTER_DATA_PORT)
}

unsafe fn write_crtc_register(index: u8, value: u8) {
    write_io_port_u8(CRTC_REGISTER_SELECT_PORT, index);
    write_io_port_u8(CRTC_REGISTER_DATA_PORT, value);
}

unsafe fn read_crtc_register(index: u8) -> u8 {
    write_io_port_u8(CRTC_REGISTER_SELECT_PORT, index);
    read_io_port_u8(CRTC_REGISTER_DATA_PORT)
}

unsafe fn enable_crtc_register() {
    let initial_value = read_io_port_u8(MISC_OUTPUT_REGISTER_READ_PORT);
    write_io_port_u8(MISC_OUTPUT_REGISTER_WRITE_PORT, initial_value | 1);
}

unsafe fn unlock_crtc() {
    // This removes protection of the CRTC registers
    let initial_value = read_crtc_register(0x03);
    write_crtc_register(0x03, initial_value | 0x80);

    // Vertical sync end register
    let initial_value = read_crtc_register(0x11);
    write_crtc_register(0x11, initial_value | 0x7F);
}

unsafe fn disable_display() {
    let initial_value = read_io_port_u8(MAIN_INDEX_REGISTER_PORT);
    write_io_port_u8(MAIN_INDEX_REGISTER_PORT, initial_value & 0xDF);
}

unsafe fn enable_display() {
    let initial_value = read_io_port_u8(MAIN_INDEX_REGISTER_PORT);
    write_io_port_u8(MAIN_INDEX_REGISTER_PORT, initial_value | 0x20);
}