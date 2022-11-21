#![no_std]
#![no_main]

use core::arch::asm;
use core::fmt::Write;

mod arch;
mod drivers;
mod entry;
mod memory;
mod std;
use crate::entry::BootInfo;
use drivers::*;
use video::{Color, TextBuffer};

fn main() {
    kprintln!("Hello from magical kernel land!");
}
