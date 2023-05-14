use core::arch::asm;
use core::fmt::Write;

#[link(name = "bios")]
extern "cdecl" {
    fn _BIOS_Video_WriteCharTeletype(c: u8);
}

static mut PRINTER: Printer = Printer {};

struct Printer {}

impl Printer {
    fn print_str(&self, s: &str) {
        for c in s.chars() {
            self.print_char(c);
        }
    }

    fn print_char(&self, c: char) {
        if c == '\n' {
            self.raw_print_char('\r');
            self.raw_print_char('\n');
        } else {
            self.raw_print_char(c);
        }
    }

    fn raw_print_char(&self, c: char) {
        unsafe {
            _BIOS_Video_WriteCharTeletype(c as u8);
        }
    }
}

impl Write for Printer {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.print_str(s);

        Ok(())
    }

    fn write_char(&mut self, c: char) -> core::fmt::Result {
        self.print_char(c);

        Ok(())
    }
}

pub fn _print(args: core::fmt::Arguments) {
    unsafe {
        PRINTER.write_fmt(args).unwrap();
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::printing::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}
