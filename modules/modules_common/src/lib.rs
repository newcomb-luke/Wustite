#![no_std]

#[derive(Clone, Copy)]
#[repr(u8)]
pub enum DriverType {
    PCIDriver = 0,
}
