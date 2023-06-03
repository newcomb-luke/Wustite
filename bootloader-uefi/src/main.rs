#![no_std]
#![no_main]

use uefi::prelude::*;

use core::fmt::Write;

#[entry]
fn main(_image_handle: Handle, mut system_table: SystemTable<Boot>) -> Status {
    uefi_services::init(&mut system_table).unwrap();

    writeln!(system_table.stdout(), "Hello, firmware!").unwrap();

    system_table.boot_services().stall(10_000_000);
    Status::SUCCESS
}
