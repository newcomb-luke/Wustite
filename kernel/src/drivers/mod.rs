#![allow(dead_code)]

use core::arch::asm;

pub mod ata;
pub mod cmos;
pub mod ide;
pub mod input;
pub mod nvme;
pub mod pci;
pub mod serial;
pub mod video;

pub type DriverResult = Result<(), ()>;

pub unsafe fn write_io_port_u8(port: u16, data: u8) {
    unsafe {
        asm!(
            "mov dx, {:x}",
            "mov al, {}",
            "out dx, al",
            in(reg) port,
            in(reg_byte) data,
            out("edx") _,
            out("eax") _,
        );
    }
}

pub unsafe fn read_io_port_u8(port: u16) -> u8 {
    let data: u8;

    unsafe {
        asm!(
            "mov dx, {:x}",
            "in al, dx",
            "mov {}, al",
            in(reg) port,
            lateout(reg_byte) data,
            out("edx") _,
            out("eax") _,
        );
    }

    data
}

pub unsafe fn write_io_port_u16(port: u16, data: u16) {
    unsafe {
        asm!(
            "mov dx, {:x}",
            "mov ax, {:x}",
            "out dx, ax",
            in(reg) port,
            in(reg) data,
            out("edx") _,
            out("eax") _,
        );
    }
}

pub unsafe fn read_io_port_u16(port: u16) -> u16 {
    let data: u16;

    unsafe {
        asm!(
            "mov dx, {:x}",
            "in ax, dx",
            "mov {:x}, ax",
            in(reg) port,
            lateout(reg) data,
            out("edx") _,
            out("eax") _,
        );
    }

    data
}

pub unsafe fn write_io_port_u32(port: u16, data: u32) {
    unsafe {
        asm!(
            "mov dx, {:x}",
            "mov eax, {:e}",
            "out dx, eax",
            in(reg) port,
            in(reg) data,
            out("edx") _,
            out("eax") _,
        );
    }
}

pub unsafe fn read_io_port_u32(port: u16) -> u32 {
    let data: u32;

    unsafe {
        asm!(
            "mov dx, {:x}",
            "in eax, dx",
            "mov {:e}, eax",
            in(reg) port,
            lateout(reg) data,
            out("edx") _,
            out("eax") _,
        );
    }

    data
}
