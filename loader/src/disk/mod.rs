use crate::{port_in, port_out};

const IO_PORT: u16 = 0x01f0;
const CONTROL_PORT: u16 = 0x3f6;

#[repr(u16)]
#[derive(Copy, Clone)]
enum ReadableIORegister {
    Data = IO_PORT,
    Error = IO_PORT + 1,
    SectorCount = IO_PORT + 2,
    SectorNumberOrLBALow = IO_PORT + 3,
    CylinderLowOrLBAMid = IO_PORT + 4,
    CylinderHighOrLBAHi = IO_PORT + 5,
    DriveAndHead = IO_PORT + 6,
    Status = IO_PORT + 7,
}

#[repr(u16)]
#[derive(Copy, Clone)]
enum WriteableIORegister {
    Data = IO_PORT,
    Features = IO_PORT + 1,
    SectorCount = IO_PORT + 2,
    SectorNumberOrLBALow = IO_PORT + 3,
    CylinderLowORLBAMid = IO_PORT + 4,
    CylinderHighORLBAHi = IO_PORT + 5,
    DriveAndHead = IO_PORT + 6,
    Command = IO_PORT + 7,
}

#[repr(u16)]
#[derive(Copy, Clone)]
enum ReadableControlRegister {
    AlternateStatus = CONTROL_PORT,
    DeviceAddress = CONTROL_PORT + 1,
}

#[repr(u16)]
#[derive(Copy, Clone)]
enum WriteableControlRegister {
    DeviceControl = CONTROL_PORT,
}

#[repr(u8)]
#[derive(Copy, Clone)]
enum DiskError {
    AddressMarkNotFound = 1,
    TrackZeroNotFound = 2,
    AbortedCommand = 4,
    MediaChangeRequest = 8,
    IDNotFound = 16,
    MediaChanged = 32,
    UncorrectableDataError = 64,
    BadBlock = 128,
}

#[repr(u8)]
#[derive(Copy, Clone)]
enum DriveNumber {
    Master = 0,
    Slave = 1,
}

#[repr(u8)]
#[derive(Copy, Clone)]
enum Command {
    Identify = 0xec,
}

#[inline]
fn send_command(cmd: Command) {
    write_to_io(WriteableIORegister::Command, cmd as u8);
}

#[inline]
fn select_drive(drive: DriveNumber) {
    write_to_io(WriteableIORegister::DriveAndHead, drive as u8);
}

#[inline]
fn set_sector_count(count: u8) {
    write_to_io(WriteableIORegister::SectorCount, count);
}

#[derive(Copy, Clone)]
struct Status {
    err: bool,
    data_ready: bool,
    srv: bool,
    drive_fault: bool,
    ready: bool,
    busy: bool,
}

impl From<u8> for Status {
    fn from(data: u8) -> Self {
        Status {
            err: data & 1 != 0,
            data_ready: data & 1 << 3 != 0,
            srv: data & 1 << 4 != 0,
            drive_fault: data & 1 << 5 != 0,
            ready: data & 1 << 6 != 0,
            busy: data & 1 << 7 != 0,
        }
    }
}

#[inline]
pub fn read_status_raw() -> u8 {
    read_from_io(ReadableIORegister::Status)
}

fn read_status() -> Status {
    let data = read_from_io(ReadableIORegister::Status);

    Status::from(data)
}

#[derive(Clone, Copy)]
enum IdentifyError {
    DriveNotExist,
    ErrSet,
    NotATA,
}

const IDENTIFY_DATA_LOC: *mut u16 = 0x01000000 as *mut u16;
const IDENTIFY_DATA_SIZE_U16S: usize = 256;

fn idenfity(drive: DriveNumber) -> Result<(), IdentifyError> {
    select_drive(drive);
    set_sector_count(0);
    write_to_io(WriteableIORegister::SectorNumberOrLBALow, 0);
    write_to_io(WriteableIORegister::CylinderLowORLBAMid, 0);
    write_to_io(WriteableIORegister::CylinderHighORLBAHi, 0);

    send_command(Command::Identify);

    // Poor man's delay
    read_status_raw();
    read_status_raw();
    read_status_raw();
    read_status_raw();
    read_status_raw();

    if read_status_raw() == 0 {
        return Err(IdentifyError::DriveNotExist);
    }

    wait_until_not_busy();

    if read_from_io(ReadableIORegister::CylinderLowOrLBAMid) != 0
        || read_from_io(ReadableIORegister::CylinderHighOrLBAHi) != 0
    {
        return Err(IdentifyError::NotATA);
    }

    wait_until_ready().map_err(|_| IdentifyError::ErrSet)?;

    for i in (0..IDENTIFY_DATA_SIZE_U16S).map(|i| i as isize) {
        // *IDENTIFY_DATA_LOC.offset(i) = read_from_io(ReadableIORegister::Data);
    }

    todo!();
}

fn software_reset() -> Result<(), ()> {
    write_to_control(WriteableControlRegister::DeviceControl, 0x04);
    write_to_control(WriteableControlRegister::DeviceControl, 0x00);

    read_status_raw();
    read_status_raw();
    read_status_raw();
    read_status_raw();

    wait_until_ready()
}

fn wait_until_ready() -> Result<(), ()> {
    let mut status = read_status();

    while status.busy || !status.ready {
        status = read_status();

        if status.err {
            return Err(());
        }
    }

    Ok(())
}

fn wait_until_not_busy() {
    let mut status = read_status();

    while status.busy {
        status = read_status();
    }
}

pub unsafe fn read_disk(cylinder: u16, head: u8, sector: u8) {}

#[inline]
fn write_to_io(port: WriteableIORegister, data: u8) {
    port_out(port as u16, data);
}

#[inline]
fn read_from_io(port: ReadableIORegister) -> u8 {
    port_in(port as u16)
}

#[inline]
fn write_to_control(port: WriteableControlRegister, data: u8) {
    port_out(port as u16, data)
}
