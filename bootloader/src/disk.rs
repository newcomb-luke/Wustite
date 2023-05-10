use core::arch::asm;

use crate::println;

#[derive(Debug)]
pub struct Disk {
    drive_number: u8,
    drive_type: u8,
    max_head: u8,
    max_cylinder: u16,
    max_sector: u8,
}

impl Disk {
    pub fn from_drive(drive_number: u8) -> Result<Self, ()> {
        get_bios_drive_params(drive_number)
    }

    pub fn drive_number(&self) -> u8 {
        self.drive_number
    }

    pub fn drive_type(&self) -> u8 {
        self.drive_type
    }

    pub fn max_head(&self) -> u8 {
        self.max_head
    }

    pub fn max_cylinder(&self) -> u16 {
        self.max_cylinder
    }

    pub fn max_sector(&self) -> u8 {
        self.max_sector
    }
}

pub fn get_bios_drive_params(drive_number: u8) -> Result<Disk, ()> {
    // BL = drive type (AT/PS2 floppies only) (see #00242)
    // CH = low eight bits of maximum cylinder number
    // CL = maximum sector number (bits 5-0)
    // high two bits of maximum cylinder number (bits 7-6)
    // DH = maximum head number
    // DL = number of drives

    let drive_type: u8;
    let ch: u8;
    let cl: u8;
    let dh: u8;
    let success: u8;

    unsafe {
        asm!(
            "push es",
             "push di",
             "xor di, di",
             "mov es, di",
             "int 0x13",
             "pop di",
             "pop es",
             "jc 3f",
             // Success
             "2: xor ax, ax",
             "jmp 4f",
             // Failure
             "3: mov ax, 1",
             "4: nop",
             in("ah") 8u8,
             in("dx") drive_number as u16,
             lateout("bl") drive_type,
             lateout("ch") ch,
             lateout("cl") cl,
             lateout("dh") dh,
             lateout("al") success
        );
    }

    println!("ch: {}", ch);

    if success != 0 {
        return Err(());
    }

    let max_cylinder: u16 = ch as u16 | ((cl as u16) & 0b11000000);
    let max_head: u8 = dh;
    let max_sector: u8 = cl & 0b00111111;

    Ok(Disk {
        drive_number,
        drive_type,
        max_head,
        max_cylinder,
        max_sector,
    })
}
