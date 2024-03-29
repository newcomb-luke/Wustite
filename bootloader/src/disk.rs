#![allow(dead_code)]

use crate::println;

const DISK_DRIVER_READ_BUFFER: *mut u8 = 0x00007E00 as *mut u8;
pub const SECTOR_SIZE: usize = 512;

#[link(name = "bios")]
extern "cdecl" {
    fn _BIOS_Drive_Reset(drive_number: u8) -> u16;

    fn _BIOS_Drive_GetParams(
        drive_number: u8,
        drive_type: *mut u8,
        max_head: *mut u8,
        max_cylinder: *mut u16,
        max_sector: *mut u8,
    ) -> u16;

    fn _BIOS_Drive_ReadSectors(
        drive_number: u8,
        head: u8,
        cylinder: u16,
        sector: u8,
        sector_count: u8,
        data_destination: *mut u8,
    ) -> u16;
}

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
        let success = unsafe { _BIOS_Drive_Reset(self.drive_number) };

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
        let success = unsafe {
            _BIOS_Drive_ReadSectors(
                self.drive_number,
                chs.head,
                chs.cylinder,
                chs.sector,
                1,
                DISK_DRIVER_READ_BUFFER,
            )
        };

        if success == 0 {
            Ok(())
        } else {
            Err(())
        }
    }

    pub fn from_drive(drive_number: u8) -> Result<Self, ()> {
        let mut drive_type = 0;
        let mut max_head = 0;
        let mut max_cylinder = 0;
        let mut max_sector = 0;

        let success = unsafe {
            _BIOS_Drive_GetParams(
                drive_number,
                core::ptr::addr_of_mut!(drive_type),
                core::ptr::addr_of_mut!(max_head),
                core::ptr::addr_of_mut!(max_cylinder),
                core::ptr::addr_of_mut!(max_sector),
            )
        };

        println!("head: {max_head}, cylinder: {max_cylinder}, sector: {max_sector}");

        if success != 0 {
            Err(())
        } else {
            Ok(Disk {
                drive_number,
                drive_type,
                max_head,
                max_cylinder,
                max_sector,
            })
        }
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
