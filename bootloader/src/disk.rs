#![allow(dead_code)]

use core::arch::asm;

const DISK_DRIVER_READ_BUFFER: *mut u8 = 0x00007E00 as *mut u8;
pub const SECTOR_SIZE: usize = 512;

#[derive(Debug, Clone, Copy)]
pub enum DiskReadError {
    DiskResetFailed,
    DiskSectorReadFailed,
}

pub struct Disk {
    drive_number: u8,
    drive_type: u8,
    max_head: u8,
    max_cylinder: u16,
    max_sector: u8,
}

#[derive(Clone, Copy)]
struct CHS {
    pub cylinder: u16,
    pub head: u8,
    pub sector: u8,
}

impl Disk {
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

    pub fn reset(&mut self) -> Result<(), ()> {
        let success: u16;

        unsafe {
            asm!(
                "stc",
                "int 0x13",
                 "jc 3f",
                 // Success
                 "2: xor ax, ax",
                 "jmp 4f",
                 // Failure
                 "3: mov ax, 1",
                 "4: nop",
                in("ah") 0x00u8,
                in("dl") self.drive_number,
                lateout("ax") success
            );
        }

        if success == 0 {
            Ok(())
        } else {
            Err(())
        }
    }

    pub fn read_sectors(
        &mut self,
        lba: u32,
        num_sectors: u32,
        destination: *mut u8,
    ) -> Result<(), DiskReadError> {
        let mut sector_destination = destination;

        for i in 0..num_sectors {
            self.read_sector(lba + i, sector_destination)?;

            unsafe {
                sector_destination = sector_destination.offset(SECTOR_SIZE as isize);
            }
        }

        Ok(())
    }

    pub fn read_sector(&mut self, lba: u32, destination: *mut u8) -> Result<(), DiskReadError> {
        let chs = self.lba_to_chs(lba);

        // The documentation on the BIOS routines say to try three times
        for _ in 0..3 {
            if self.bios_read_sector(chs).is_ok() {
                unsafe {
                    core::ptr::copy_nonoverlapping(
                        DISK_DRIVER_READ_BUFFER,
                        destination,
                        SECTOR_SIZE,
                    );
                }

                return Ok(());
            }

            self.reset().map_err(|_| DiskReadError::DiskResetFailed)?;
        }

        Err(DiskReadError::DiskSectorReadFailed)
    }

    fn bios_read_sector(&mut self, chs: CHS) -> Result<(), ()> {
        // AH = 0x02
        // AL = number of sectors to read (must be nonzero)
        // CH = low eight bits of cylinder number
        // CL = sector number 1-63 (bits 0-5)
        // high two bits of cylinder (bits 6-7, hard disk only)
        // DH = head number
        // DL = drive number (bit 7 set for hard disk)
        // ES:BX -> data buffer

        let ch = (chs.cylinder & 0x00FF) as u8;
        let cl_sector = chs.sector & 0b00111111;
        let cl_cylinder = (((chs.cylinder >> 8) & 0x03) << 6) as u8;
        let cl = cl_sector | cl_cylinder;
        let bx = DISK_DRIVER_READ_BUFFER as u16;

        let success: u16;

        unsafe {
            asm!(
                "push es",
                "stc",
                "int 0x13",
                "pop es",
                "jc 3f",
                // Success
                "2: xor ax, ax",
                "jmp 4f",
                // Failure
                "3: mov ax, 1",
                "4: nop",
                in("ah") 0x02u8,
                in("al") 1u8,
                in("ch") ch,
                in("cl") cl,
                in("dh") chs.head,
                in("dl") self.drive_number,
                in("bx") bx,
                lateout("ax") success,
            );
        }

        if success == 0 {
            Ok(())
        } else {
            Err(())
        }
    }

    pub fn from_drive(drive_number: u8) -> Result<Self, ()> {
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
        let success: u16;

        unsafe {
            asm!(
                 "push es",
                 "push di",
                 "xor di, di",
                 "mov es, di",
                 "stc",
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
                 lateout("ax") success
            );
        }

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

    /// Converts a LBA sector address to a CHS address
    fn lba_to_chs(&self, lba: u32) -> CHS {
        // sector = (LBA % sectors per cylinder + 1)
        let sector = ((lba % (self.max_sector as u32)) + 1) as u8;
        // cylinder = (LBA / sectors per cylinder) / heads on disk
        let cylinder = (lba / ((self.max_sector as u32) * (self.max_head as u32 + 1))) as u16;
        // head = (LBA / sectors per cylinder) % heads on disk
        let head = ((lba / (self.max_sector as u32)) % (self.max_head as u32 + 1)) as u8;

        CHS {
            cylinder,
            head,
            sector,
        }
    }
}
